package persistence

import (
	"context"
	"errors"
	"sync"

	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// SequentialWriter executes operations sequentially across all writers.
type SequentialWriter struct{}

// ParallelWriter executes operations concurrently across writers.
type ParallelWriter struct{}

type writerResult struct {
	writer    spi.CacheWriterPort
	committed []PersistenceOperation
	err       error
}

// Name returns the strategy name.
func (s *SequentialWriter) Name() string { return "Sequential" }

// Describe returns a description of the strategy.
func (s *SequentialWriter) Describe() string {
	return "Executes operations sequentially across all writers"
}

// Execute performs the operations sequentially on each writer.
func (s *SequentialWriter) Execute(
	ctx context.Context,
	ops []PersistenceOperation,
	writers []spi.CacheWriterPort,
) error {
	for _, writer := range writers {
		for i := range ops {
			if err := ctx.Err(); err != nil {
				return err
			}
			if err := ops[i].Execute(ctx, writer); err != nil {
				return err
			}
		}
	}
	return nil
}

// Name returns the strategy name.
func (s *ParallelWriter) Name() string { return "Parallel" }

// Describe returns a description of the strategy.
func (s *ParallelWriter) Describe() string {
	return "Executes operations concurrently across writers with coordinated rollback"
}

// Execute performs the operations in parallel on each writer.
func (s *ParallelWriter) Execute(
	ctx context.Context,
	ops []PersistenceOperation,
	writers []spi.CacheWriterPort,
) error {
	if len(writers) == 0 {
		return nil
	}

	results := make(chan writerResult, len(writers))
	var wg sync.WaitGroup

	for i := range writers {
		w := writers[i]
		wg.Add(1)
		go func(writer spi.CacheWriterPort) {
			defer wg.Done()
			var committed []PersistenceOperation
			for j := range ops {
				op := ops[j]
				if err := op.Execute(ctx, writer); err != nil {
					results <- writerResult{writer: writer, committed: committed, err: err}
					return
				}
				committed = append(committed, op)
			}
			results <- writerResult{writer: writer, committed: committed, err: nil}
		}(w)
	}

	wg.Wait()
	close(results)

	var combinedErr error
	var allResults []writerResult
	for res := range results {
		allResults = append(allResults, res)
		if res.err != nil {
			combinedErr = errors.Join(combinedErr, res.err)
		}
	}

	if combinedErr != nil {
		for i := range allResults {
			res := allResults[i]
			s.rollback(ctx, res.committed, res.writer)
		}
	}

	return combinedErr
}

func (s *ParallelWriter) rollback(
	ctx context.Context,
	ops []PersistenceOperation,
	writer spi.CacheWriterPort,
) {
	for i := len(ops) - 1; i >= 0; i-- {
		_ = ops[i].Rollback(ctx, writer)
	}
}
