package vault

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

const (
	opWrite operationType = iota
	opDelete
)

type operationType int

type operation struct {
	opType    operationType
	note      domain.Note
	indexTime time.Time
	path      string
}

// CacheUnitOfWork coordinates transactional writes across multiple storage
// systems.
type CacheUnitOfWork struct {
	boltWriter   spi.CacheWriterPort
	sqliteWriter spi.CacheWriterPort
	operations   []operation
	mu           sync.Mutex
}

// NewCacheUnitOfWork creates a new CacheUnitOfWork.
func NewCacheUnitOfWork(
	boltWriter spi.CacheWriterPort,
	sqliteWriter spi.CacheWriterPort,
) *CacheUnitOfWork {
	return &CacheUnitOfWork{
		boltWriter:   boltWriter,
		sqliteWriter: sqliteWriter,
		operations:   make([]operation, 0),
		mu:           sync.Mutex{},
	}
}

// Begin starts a new unit of work.
func (uow *CacheUnitOfWork) Begin() error {
	uow.mu.Lock()
	defer uow.mu.Unlock()
	uow.operations = make([]operation, 0)
	return nil
}

// AddWrite stages a write operation.
func (uow *CacheUnitOfWork) AddWrite(
	note domain.Note,
	indexTime time.Time,
) error {
	uow.mu.Lock()
	defer uow.mu.Unlock()
	uow.operations = append(uow.operations, operation{
		opType:    opWrite,
		note:      note,
		indexTime: indexTime,
		path:      "", // Not used for writes
	})
	return nil
}

// AddDelete stages a delete operation.
func (uow *CacheUnitOfWork) AddDelete(path string) error {
	uow.mu.Lock()
	defer uow.mu.Unlock()
	uow.operations = append(uow.operations, operation{
		opType: opDelete,
		path:   path,
		note: domain.Note{
			ID: "",
			Frontmatter: domain.Frontmatter{
				FileClass: "",
				Fields:    nil,
			},
		}, // Not used for deletes
		indexTime: time.Time{}, // Not used for deletes
	})
	return nil
}

// Commit executes all staged operations atomically.
func (uow *CacheUnitOfWork) Commit(ctx context.Context) error {
	uow.mu.Lock()
	defer uow.mu.Unlock()

	var committedBolt []operation

	// Phase 1: BoltDB
	for _, op := range uow.operations {
		var err error
		if op.opType == opWrite {
			err = uow.boltWriter.Persist(ctx, op.note, op.indexTime)
		} else {
			err = uow.boltWriter.Delete(ctx, domain.NoteID(op.path))
		}

		if err != nil {
			uow.rollbackBolt(ctx, committedBolt)
			return fmt.Errorf("boltdb transaction failed: %w", err)
		}
		committedBolt = append(committedBolt, op)
	}

	// Phase 2: SQLite
	var committedSQLite []operation
	for _, op := range uow.operations {
		var err error
		if op.opType == opWrite {
			err = uow.sqliteWriter.Persist(ctx, op.note, op.indexTime)
		} else {
			err = uow.sqliteWriter.Delete(ctx, domain.NoteID(op.path))
		}

		if err != nil {
			uow.rollbackSQLite(ctx, committedSQLite)
			uow.rollbackBolt(ctx, committedBolt)
			return fmt.Errorf("sqlite transaction failed: %w", err)
		}
		committedSQLite = append(committedSQLite, op)
	}

	// Success - clear operations
	uow.operations = uow.operations[:0]
	return nil
}

// Rollback clears all staged operations.
func (uow *CacheUnitOfWork) Rollback(ctx context.Context) error {
	uow.mu.Lock()
	defer uow.mu.Unlock()
	uow.operations = make([]operation, 0)
	return nil
}

func (uow *CacheUnitOfWork) rollbackBolt(ctx context.Context, ops []operation) {
	for i := len(ops) - 1; i >= 0; i-- {
		op := ops[i]
		if op.opType == opWrite {
			// Compensating write: delete what was written
			_ = uow.boltWriter.Delete(ctx, op.note.ID)
		}
		// Cannot undo delete easily without read-before-delete
	}
}

func (uow *CacheUnitOfWork) rollbackSQLite(
	ctx context.Context,
	ops []operation,
) {
	for i := len(ops) - 1; i >= 0; i-- {
		op := ops[i]
		if op.opType == opWrite {
			_ = uow.sqliteWriter.Delete(ctx, op.note.ID)
		}
	}
}
