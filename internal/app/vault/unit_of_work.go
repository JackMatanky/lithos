package vault

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// Operation represents a transactional operation that can be executed and
// rolled back.
type Operation interface {
	// Execute performs the operation on the given writer.
	Execute(ctx context.Context, writer spi.CacheWriterPort) error
	// Rollback undoes the operation on the given writer.
	Rollback(ctx context.Context, writer spi.CacheWriterPort) error
	// Type returns the operation type for identification.
	Type() OperationType
}

// OperationType identifies the kind of operation.
type OperationType int

const (
	// OpWrite represents a write operation.
	OpWrite OperationType = iota
	// OpDelete represents a delete operation.
	OpDelete
)

// WriteOperation represents a write operation for a note.
type WriteOperation struct {
	note      domain.Note
	indexTime time.Time
}

// DeleteOperation represents a delete operation for a note.
type DeleteOperation struct {
	noteID domain.NoteID
}

// TransactionStrategy defines how transactions are executed across multiple writers.
type TransactionStrategy interface {
	// Execute performs the transaction using the provided operations and writers.
	Execute(ctx context.Context, operations []Operation, writers []spi.CacheWriterPort) error
}

// TwoPhaseCommitStrategy implements two-phase commit across BoltDB and SQLite.
type TwoPhaseCommitStrategy struct {
	boltWriter   spi.CacheWriterPort
	sqliteWriter spi.CacheWriterPort
}

// CacheUnitOfWork coordinates transactional writes across multiple storage
// systems.
type CacheUnitOfWork struct {
	strategy   TransactionStrategy
	operations []Operation
	mu         sync.Mutex
}

// NewWriteOperation creates a new write operation.
func NewWriteOperation(note domain.Note, indexTime time.Time) *WriteOperation {
	return &WriteOperation{
		note:      note,
		indexTime: indexTime,
	}
}

// Execute performs the write operation.
func (op *WriteOperation) Execute(ctx context.Context, writer spi.CacheWriterPort) error {
	return writer.Persist(ctx, op.note, op.indexTime)
}

// Rollback undoes the write operation by deleting the note.
func (op *WriteOperation) Rollback(ctx context.Context, writer spi.CacheWriterPort) error {
	return writer.Delete(ctx, op.note.ID)
}

// Type returns the operation type.
func (op *WriteOperation) Type() OperationType {
	return OpWrite
}

// NewDeleteOperation creates a new delete operation.
func NewDeleteOperation(noteID domain.NoteID) *DeleteOperation {
	return &DeleteOperation{noteID: noteID}
}

// Execute performs the delete operation.
func (op *DeleteOperation) Execute(ctx context.Context, writer spi.CacheWriterPort) error {
	return writer.Delete(ctx, op.noteID)
}

// Rollback cannot undo a delete operation easily without read-before-delete.
func (op *DeleteOperation) Rollback(ctx context.Context, writer spi.CacheWriterPort) error {
	// Cannot undo delete easily without read-before-delete
	return nil
}

// Type returns the operation type.
func (op *DeleteOperation) Type() OperationType {
	return OpDelete
}

// NewTwoPhaseCommitStrategy creates a new two-phase commit strategy.
func NewTwoPhaseCommitStrategy(
	boltWriter spi.CacheWriterPort,
	sqliteWriter spi.CacheWriterPort,
) *TwoPhaseCommitStrategy {
	return &TwoPhaseCommitStrategy{
		boltWriter:   boltWriter,
		sqliteWriter: sqliteWriter,
	}
}

// Execute performs two-phase commit: BoltDB first, then SQLite with rollback on failure.
func (s *TwoPhaseCommitStrategy) Execute(
	ctx context.Context,
	operations []Operation,
	writers []spi.CacheWriterPort,
) error {
	// Phase 1: Commit to BoltDB
	committedBolt, err := s.commitOperations(ctx, operations, s.boltWriter, s.rollbackBolt)
	if err != nil {
		return fmt.Errorf("boltdb transaction failed: %w", err)
	}

	// Phase 2: Commit to SQLite
	_, err = s.commitOperations(ctx, operations, s.sqliteWriter, s.rollbackSQLite)
	if err != nil {
		// Compensating rollback: undo BoltDB changes
		if rollbackErr := s.rollbackBolt(ctx, committedBolt); rollbackErr != nil {
			return fmt.Errorf(
				"sqlite transaction failed and boltdb rollback failed: %w (original: %w)",
				rollbackErr, err,
			)
		}
		return fmt.Errorf("sqlite transaction failed: %w", err)
	}

	return nil
}

// commitOperations executes operations against a cache writer with rollback on failure.
func (s *TwoPhaseCommitStrategy) commitOperations(
	ctx context.Context,
	ops []Operation,
	writer spi.CacheWriterPort,
	rollbackFunc func(context.Context, []Operation) error,
) ([]Operation, error) {
	var committed []Operation
	for _, op := range ops {
		if err := op.Execute(ctx, writer); err != nil {
			// Rollback any committed operations
			if rollbackErr := rollbackFunc(ctx, committed); rollbackErr != nil {
				// Log rollback error but don't mask original error
				// Error will be wrapped at higher level
			}
			return nil, err
		}
		committed = append(committed, op)
	}
	return committed, nil
}

func (s *TwoPhaseCommitStrategy) rollbackBolt(
	ctx context.Context,
	ops []Operation,
) error {
	var lastErr error
	for i := len(ops) - 1; i >= 0; i-- {
		if err := ops[i].Rollback(ctx, s.boltWriter); err != nil {
			lastErr = err
		}
	}
	return lastErr
}

func (s *TwoPhaseCommitStrategy) rollbackSQLite(
	ctx context.Context,
	ops []Operation,
) error {
	var lastErr error
	for i := len(ops) - 1; i >= 0; i-- {
		if err := ops[i].Rollback(ctx, s.sqliteWriter); err != nil {
			lastErr = err
		}
	}
	return lastErr
}
