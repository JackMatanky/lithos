package events

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"sync"
	"sync/atomic"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
)

const (
	defaultWorkerCount = 4
	defaultBufferSize  = 64
)

var (
	errInvalidEventType = errors.New("event type is required")
	errNilHandler       = errors.New("handler must not be nil")
	errBusClosed        = errors.New("event bus has been shut down")
)

// EventHandler reacts to a published DomainEvent.
type EventHandler func(ctx context.Context, event domain.DomainEvent) error

// EventBus defines the publish/subscribe contract.
type EventBus interface {
	Publish(ctx context.Context, event domain.DomainEvent) error
	Subscribe(eventType string, handler EventHandler) error
	Unsubscribe(eventType string, handler EventHandler) error
	Shutdown(ctx context.Context) error
}

// Option configures the in-memory event bus.
type Option func(*inMemoryEventBus)

// inMemoryEventBus is a goroutine-based pub/sub implementation.
type inMemoryEventBus struct {
	log          logger.Logger
	workerCount  int
	bufferSize   int
	subscribers  map[string][]handlerRecord
	mu           sync.RWMutex
	events       chan eventEnvelope
	workerWG     sync.WaitGroup
	shutdownOnce sync.Once
	closed       atomic.Bool
}

type handlerRecord struct {
	key uintptr
	fn  EventHandler
}

// eventEnvelope carries publish metadata to the worker pool.
type eventEnvelope struct {
	ctx    context.Context
	event  domain.DomainEvent
	result chan error
}

// WithWorkerCount sets the number of dispatch workers.
func WithWorkerCount(count int) Option {
	return func(bus *inMemoryEventBus) {
		if count > 0 {
			bus.workerCount = count
		}
	}
}

// WithBufferSize sets the event queue buffer size.
func WithBufferSize(size int) Option {
	return func(bus *inMemoryEventBus) {
		if size > 0 {
			bus.bufferSize = size
		}
	}
}

func newEnvelope(ctx context.Context, event domain.DomainEvent) eventEnvelope {
	return eventEnvelope{ctx: ctx, event: event, result: make(chan error, 1)}
}

func (e eventEnvelope) respond(err error) {
	e.result <- err
	close(e.result)
}

// NewInMemoryEventBus creates an EventBus backed by goroutines.
func NewInMemoryEventBus(log logger.Logger, opts ...Option) EventBus {
	bus := &inMemoryEventBus{
		log:          log,
		workerCount:  defaultWorkerCount,
		bufferSize:   defaultBufferSize,
		subscribers:  make(map[string][]handlerRecord),
		mu:           sync.RWMutex{},
		events:       nil,
		workerWG:     sync.WaitGroup{},
		shutdownOnce: sync.Once{},
		closed:       atomic.Bool{},
	}
	for _, opt := range opts {
		opt(bus)
	}
	bus.events = make(chan eventEnvelope, bus.bufferSize)
	bus.startWorkers()
	return bus
}

// Publish dispatches an event to all registered subscribers.
func (b *inMemoryEventBus) Publish(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	if event == nil {
		return fmt.Errorf("event is nil")
	}
	if b.closed.Load() {
		return errBusClosed
	}
	if event.EventType() == "" {
		return errInvalidEventType
	}
	envelope := newEnvelope(ctx, event)

	select {
	case <-ctx.Done():
		return ctx.Err()
	case b.events <- envelope:
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case err := <-envelope.result:
		return err
	}
}

// Subscribe registers a handler for a specific event type.
func (b *inMemoryEventBus) Subscribe(
	eventType string,
	handler EventHandler,
) error {
	if eventType == "" {
		return errInvalidEventType
	}
	if handler == nil {
		return errNilHandler
	}
	b.mu.Lock()
	defer b.mu.Unlock()
	record := handlerRecord{key: handlerKey(handler), fn: handler}
	b.subscribers[eventType] = append(b.subscribers[eventType], record)
	return nil
}

// Unsubscribe removes a handler for a specific event type.
func (b *inMemoryEventBus) Unsubscribe(
	eventType string,
	handler EventHandler,
) error {
	if eventType == "" {
		return errInvalidEventType
	}
	if handler == nil {
		return errNilHandler
	}
	b.mu.Lock()
	defer b.mu.Unlock()
	records := b.subscribers[eventType]
	key := handlerKey(handler)
	for i := range records {
		if records[i].key == key {
			records = append(records[:i], records[i+1:]...)
			break
		}
	}
	if len(records) == 0 {
		delete(b.subscribers, eventType)
	} else {
		b.subscribers[eventType] = records
	}
	return nil
}

// Shutdown gracefully stops the event bus and waits for workers to finish.
func (b *inMemoryEventBus) Shutdown(ctx context.Context) error {
	var err error
	b.shutdownOnce.Do(func() {
		b.closed.Store(true)
		close(b.events)
	})

	ch := make(chan struct{})
	go func() {
		b.workerWG.Wait()
		close(ch)
	}()

	select {
	case <-ctx.Done():
		err = ctx.Err()
	case <-ch:
		err = nil
	}

	return err
}

func (b *inMemoryEventBus) startWorkers() {
	for range b.workerCount {
		b.workerWG.Add(1)
		go func() {
			defer b.workerWG.Done()
			for envelope := range b.events {
				b.dispatch(envelope)
			}
		}()
	}
}

func (b *inMemoryEventBus) dispatch(envelope eventEnvelope) {
	handlers := b.snapshotHandlers(envelope.event.EventType())
	if len(handlers) == 0 {
		envelope.respond(nil)
		return
	}

	var wg sync.WaitGroup
	errCh := make(chan error, len(handlers))
	for _, handler := range handlers {
		h := handler
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := h(envelope.ctx, envelope.event); err != nil {
				errCh <- err
			}
		}()
	}

	wg.Wait()
	close(errCh)

	var combined error
	for err := range errCh {
		combined = errors.Join(combined, err)
	}

	if combined != nil {
		b.log.WithFields(map[string]interface{}{
			"event_type":   envelope.event.EventType(),
			"aggregate_id": envelope.event.AggregateID(),
		}).WithError(combined).Error("event handler failures")
	}
	// AC 9: Failed handlers don't block other subscribers - always respond with
	// nil
	envelope.respond(nil)
}

func (b *inMemoryEventBus) snapshotHandlers(eventType string) []EventHandler {
	b.mu.RLock()
	defer b.mu.RUnlock()
	records := b.subscribers[eventType]
	if len(records) == 0 {
		return nil
	}
	handlers := make([]EventHandler, len(records))
	for i, record := range records {
		handlers[i] = record.fn
	}
	return handlers
}

func handlerKey(handler EventHandler) uintptr {
	return reflect.ValueOf(handler).Pointer()
}
