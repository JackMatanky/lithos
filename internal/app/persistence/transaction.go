package persistence

import (
	"context"
	"fmt"
	"sync"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

const (
	// OpWrite represents a write/update operation.
	OpWrite OperationType = iota
	// OpDelete represents a removal operation.
	OpDelete
)

// OperationType identifies the kind of persistence operation.
type OperationType int

// PersistenceOperation represents a single unit of work for persistence.
type PersistenceOperation interface {
	// Execute performs the operation on the given writer.
	Execute(ctx context.Context, writer spi.CacheWriterPort) error
	// Rollback undoes the operation on the given writer.
	Rollback(ctx context.Context, writer spi.CacheWriterPort) error
	// Type returns the operation type.
	Type() OperationType
}

// WriteStrategy defines how operations are executed across multiple writers.
type WriteStrategy interface {
	// Execute performs the transaction using provided ops and writers.
	Execute(
		ctx context.Context,
		operations []PersistenceOperation,
		writers []spi.CacheWriterPort,
	) error
	// Name returns the strategy identifier.
	Name() string
	// Describe returns a human-readable description of the strategy.
	Describe() string
}

// WriteOperation implements PersistenceOperation for writing a note.
type WriteOperation struct {
	Note     domain.Note
	Metadata spi.CacheWriteMetadata
}

// DeleteOperation implements PersistenceOperation for removing a note.
type DeleteOperation struct {
	NotePath string
}

// CacheTransaction coordinates transactional writes across storage systems.
type CacheTransaction struct {
	writers    []spi.CacheWriterPort
	strategy   WriteStrategy
	operations []PersistenceOperation
	mu         sync.Mutex
}

// NewCacheTransaction creates a new transaction coordinator.
func NewCacheTransaction(
	strategy WriteStrategy,
	writers ...spi.CacheWriterPort,
) *CacheTransaction {
	return &CacheTransaction{
		writers:    writers,
		strategy:   strategy,
		operations: make([]PersistenceOperation, 0),
		mu:         sync.Mutex{},
	}
}

// Execute performs the write operation.
func (op WriteOperation) Execute(
	ctx context.Context,
	writer spi.CacheWriterPort,
) error {
	return writer.Persist(ctx, op.Note, op.Metadata)
}

// Rollback undoes the write operation by deleting the note.
func (op WriteOperation) Rollback(
	ctx context.Context,
	writer spi.CacheWriterPort,
) error {
	return writer.Delete(ctx, op.Note.Path)
}

// Type returns the OpWrite operation type.
func (op WriteOperation) Type() OperationType {
	return OpWrite
}

// Execute performs the delete operation.
func (op DeleteOperation) Execute(
	ctx context.Context,
	writer spi.CacheWriterPort,
) error {
	return writer.Delete(ctx, op.NotePath)
}

// Rollback undoes the delete operation.
func (op DeleteOperation) Rollback(
	_ context.Context,
	_ spi.CacheWriterPort,
) error {
	// Rollback of delete is complex without read-before-delete (backup).
	// For now, we follow the previous pattern of no-op rollback for delete.
	return nil
}

// Type returns the OpDelete operation type.
func (op DeleteOperation) Type() OperationType {
	return OpDelete
}

// AddWrite stages a note for persistence.
func (t *CacheTransaction) AddWrite(
	note domain.Note,
	metadata spi.CacheWriteMetadata,
) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.operations = append(
		t.operations,
		WriteOperation{Note: note, Metadata: metadata},
	)
}

// AddDelete stages a note for removal.
func (t *CacheTransaction) AddDelete(path string) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.operations = append(t.operations, DeleteOperation{NotePath: path})
}

// Commit executes the staged operations using the configured strategy.
func (t *CacheTransaction) Commit(ctx context.Context) error {
	t.mu.Lock()
	defer t.mu.Unlock()

	if len(t.operations) == 0 {
		return nil
	}

	staged := t.operations
	if err := t.strategy.Execute(ctx, staged, t.writers); err != nil {
		return fmt.Errorf(
			"transaction failed using %s strategy: %w",
			t.strategy.Name(),
			err,
		)
	}

	t.operations = t.operations[:0]
	return nil
}

// Rollback clears the staged operations.
func (t *CacheTransaction) Rollback() {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.operations = t.operations[:0]
}
