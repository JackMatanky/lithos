# Shared Errors Package

Lithos uses a lean error handling foundation centred on a Result[T] type and a
small set of focused error structs. This package provides those building blocks
so every layer of the application speaks a consistent error language.

## Result[T] Pattern

`Result[T]` models the outcome of computations that may fail. It mirrors Rust's
`Result` while staying idiomatic to Go.

```go
import sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"

func loadTemplate(name string) sharederrors.Result[string] {
	return sharederrors.
		Ok(name).
		Map(strings.ToUpper).
		AndThen(resolveTemplateFromDisk)
}
```

### Key Operations

```go
r := sharederrors.Ok(42)

r.IsOk()             // true
r.IsErr()            // false
r.Value()            // 42 (panics if err)
r.Error()            // nil
r.Err()              // nil (alias for Error)
r.ValueOr(7)         // 42
r = sharederrors.Map(r, func(v int) string { return strconv.Itoa(v) })
r = sharederrors.AndThen(r, func(v int) sharederrors.Result[int] { ... })
r.OrElse(func(err error) sharederrors.Result[int] { ... })
r.Inspect(func(v int) { log.Println(v) })
r.InspectErr(func(err error) { log.Println(err) })
```

- `Map` transforms successful values and propagates errors.
- `AndThen` composes operations that themselves return `Result`.
- `OrElse` lets you recover from error states.

`Err(err)` panics when `err` is `nil` to avoid silent misuse.

## Core Error Types

### BaseError

`BaseError` stores only a message and optional cause. All domain errors embed it.

```go
err := sharederrors.NewBaseError("schema registry unavailable", upstreamErr)
fmt.Println(err.Error()) // "schema registry unavailable: connection refused"
errors.Is(err, upstreamErr) // true
```

### ValidationError

Represents property-level validation failures using Lithos terminology.

```go
err := sharederrors.NewValidationError("title", "cannot be empty", "")
err.Property() // "title"
err.Reason()   // "cannot be empty"
err.Value()    // ""
fmt.Println(err) // "property 'title': cannot be empty (value: )"
```

### ResourceError

Describes failed operations against a concrete resource.

```go
err := sharederrors.NewResourceError("file", "read", "/vault/note.md", os.ErrNotExist)
err.Resource()  // "file"
err.Operation() // "read"
err.Target()    // "/vault/note.md"
fmt.Println(err) // "file read '/vault/note.md': file does not exist"
```

## Domain-Specific Errors

### Schema Errors

```go
// High-level schema problem
schemaErr := sharederrors.NewSchemaError("article", "invalid property bank", nil)

// Property-level schema validation
propErr := sharederrors.NewSchemaValidationError("article", "summary", "must be <= 140 chars", summary, nil)

// Missing schema
notFound := sharederrors.NewSchemaNotFoundError("article")
```

### Frontmatter Errors

`FrontmatterFieldError` captures issues in end-user frontmatter while preserving
field terminology.

```go
missing := sharederrors.NewRequiredFieldError("title")
arrayMismatch := sharederrors.NewArrayConstraintError("tags", "foo", "array")
fieldValidation := sharederrors.NewFieldValidationError("title", "invalid casing", "VALUE", nil)

missing.ConstraintType()     // "required"
arrayMismatch.ConstraintType() // "array"
fieldValidation.ConstraintType() // "validation"
fieldValidation.Field()   // "title"
```

Use `ValidationResult` to aggregate multiple issues:

```go
result := sharederrors.NewValidationResult()
result.AddError(missing)
result.AddError(arrayMismatch)

result.IsValid()  // false
len(result.Errors) // 2
```

### Template Errors

```go
tmplErr := sharederrors.NewTemplateError("header.md", 12, "undefined placeholder", nil)

tmplErr.Template() // "header.md"
tmplErr.Line()     // 12
fmt.Println(tmplErr) // "template 'header.md' line 12: undefined placeholder"
```

## Usage Guidelines

1. Use `Result[T]` at port boundaries and long-running workflows.
2. Return `ValidationError` (or a domain variant) for property issues.
3. Prefer `ResourceError` for IO and filesystem interactions.
4. Wrap upstream failures with `NewBaseError` or pass them as causes to domain
   constructors so `errors.Is/As` continues to work.
5. Aggregate validation issues with `ValidationResult` instead of returning
   after the first failure.
