# Shared Errors Package

This package provides domain-specific error types and functional error handling patterns for consistent error handling across the Lithos application.

## Package Purpose

The `internal/shared/errors` package serves as a centralized location for:

- **Domain-specific error types** with structured context for better debugging
- **Functional error handling** using a custom `Result[T]` pattern inspired by Rust
- **Error wrapping utilities** for adding context while preserving error chains
- **Consistent error formatting** following the pattern `[ErrorType] context: message`

This package is a **shared internal package** - a cross-cutting concern used by all application layers but not containing domain logic or infrastructure dependencies.

## Architecture Role

- **Used by**: All domain services, ports, adapters, and command handlers
- **Layer**: Shared Internal (Cross-cutting concern)
- **Dependencies**: Only Go standard library (`errors`, `fmt`)
- **Principles**: Standard Library First, Result pattern for port boundaries

## Package Structure

The package is organized across multiple files for better maintainability and Single Responsibility Principle compliance:

- **`README.md`**: Package documentation and usage guide
- **`result.go`**: Result[T] type, ResultInterface, and functional error handling methods
- **`types.go`**: BaseError composition system and domain-specific error types with constructors
- **`validation.go`**: Comprehensive validation error system with aggregation
- **`wrapping.go`**: Error wrapping and utility functions
- **`*_test.go`**: Unit tests for each component

## Error Types

### BaseError

Provides a common structure for domain errors with consistent formatting and error chaining.

```go
type BaseError struct {
    Type    string      // Error type identifier (e.g., "ValidationError")
    Context string      // Primary context information
    Detail  string      // Specific error detail
    Value   interface{} // Optional associated value
    Cause   error       // Optional underlying cause
}
```

### Validation System

For comprehensive validation with multiple field errors, see `validation.go`:

- **`ValidationResult`**: Aggregates multiple field validation errors
- **`FieldValidationError`**: Interface for polymorphic field error handling
- **Specific error types**: `RequiredFieldError`, `ArrayConstraintError`, `PropertySpecError`, etc.

**Usage:**
```go
// Single field validation
err := NewValidationError("email", "invalid format", "user@")

// Comprehensive validation with multiple errors
result := ValidateFrontmatter(frontmatter, schema)
if !result.IsValid() {
    for _, fieldErr := range result.Errors {
        log.Printf("Field %s: %v", fieldErr.Field(), fieldErr)
    }
}
```

### NotFoundError

Represents resource not found errors.

```go
type NotFoundError struct {
    Resource   string // Type of resource (e.g., "user", "note")
    Identifier string // Identifier that was not found
}
```

**Usage:**
```go
err := NewNotFoundError("user", "123")
fmt.Println(err.Error()) // "[NotFoundError] user '123' not found"
```

### ConfigurationError

Represents configuration-related errors.

```go
type ConfigurationError struct {
    Key     string // Configuration key that caused the error
    Message string // Error description
}
```

**Usage:**
```go
err := NewConfigurationError("database.url", "missing required value")
fmt.Println(err.Error()) // "[ConfigurationError] key 'database.url': missing required value"
```

### TemplateError

Represents template processing errors.

```go
type TemplateError struct {
    Template string // Template name or path
    Line     int    // Line number (0 if not applicable)
    Message  string // Error description
}
```

**Usage:**
```go
err := NewTemplateError("welcome.tmpl", 5, "undefined variable 'name'")
fmt.Println(err.Error()) // "[TemplateError] template 'welcome.tmpl' line 5: undefined variable 'name'"

err2 := NewTemplateError("header.tmpl", 0, "syntax error")
fmt.Println(err2.Error()) // "[TemplateError] template 'header.tmpl': syntax error"
```

### SchemaError

Represents schema-related errors.

```go
type SchemaError struct {
    Schema  string // Schema name or identifier
    Message string // Error description
}
```

**Usage:**
```go
err := NewSchemaError("user.schema", "invalid field type for 'age'")
fmt.Println(err.Error()) // "[SchemaError] schema 'user.schema': invalid field type for 'age'"
```

### OperationError (StorageError/FileSystemError)

Represents operation failures for storage and filesystem operations. `StorageError` and `FileSystemError` are type aliases for backward compatibility.

```go
type OperationError struct {
    Operation string // Operation that failed (e.g., "read", "write", "open")
    Path      string // Path or identifier of the location
    Cause     error  // Underlying cause (nil if not applicable)
}
```

**Usage:**
```go
// Storage operations
storageErr := NewStorageError("write", "/data/cache/users", errors.New("disk full"))
fmt.Println(storageErr.Error()) // "[StorageError] write '/data/cache/users': disk full"

readErr := NewStorageError("read", "/data/cache/users", nil)
fmt.Println(readErr.Error()) // "[StorageError] read '/data/cache/users' failed"

// Filesystem operations
fsErr := NewFileSystemError("open", "/tmp/readonly.txt", errors.New("permission denied"))
fmt.Println(fsErr.Error()) // "[FileSystemError] open '/tmp/readonly.txt': permission denied"
```

## Result[T] Pattern

The `Result[T]` type implements functional error handling similar to Rust's `Result<T>` and satisfies the `ResultInterface[T]` for polymorphic usage.

```go
type Result[T any] struct {
    value T
    err   error
}

// ResultInterface allows for different result implementations
type ResultInterface[T any] interface {
    IsOk() bool
    IsErr() bool
    Unwrap() (T, error)
    Value() T
    Error() error
}
```

### Creating Results

```go
// Success result
result := Ok("operation successful")

// Error result
result := Err[string](errors.New("operation failed"))
```

### Checking Result State

```go
if result.IsOk() {
    // Handle success case
    value := result.Value()
}

if result.IsErr() {
    // Handle error case
    err := result.Error()
}
```

### Safe Value Extraction

```go
// Unwrap both value and error (check state first!)
value, err := result.Unwrap()

// Get value only (panics if error result - check IsOk() first!)
value := result.Value()

// Get error only (returns nil if ok result)
err := result.Error()
```

### Usage in Functions

```go
func validateUser(email string) Result[User] {
    if !isValidEmail(email) {
        return Err[User](NewValidationError("email", "invalid format", email))
    }

    user, err := findUserByEmail(email)
    if err != nil {
        return Err[User](Wrap(err, "failed to find user"))
    }

    return Ok(user)
}

// Usage with concrete type
result := validateUser("user@example.com")
if result.IsOk() {
    user := result.Value()
    // Use user...
} else {
    log.Printf("Validation failed: %v", result.Error())
}

// Usage with interface (allows different implementations)
var resultInterface ResultInterface[User] = validateUser("user@example.com")
if resultInterface.IsOk() {
    user := resultInterface.Value()
    // Use user...
}
```

## Factory Functions

All error types provide factory functions for consistent creation:

```go
// Resource errors
err := NewNotFoundError("resource", "identifier")

// Configuration errors
err := NewConfigurationError("key", "message")

// Template errors
err := NewTemplateError("template", line, "message")

// Schema errors
err := NewSchemaError("schema", "message")

// Operation errors (Storage/FileSystem consolidated)
storageErr := NewStorageError("operation", "path", cause)
fsErr := NewFileSystemError("operation", "path", cause)

// Validation errors (comprehensive system)
validationErr := NewValidationError("field", "message", value)
result := ValidateFrontmatter(frontmatter, schema)
```

## Error Wrapping Utilities

### Wrap

Add simple context to an error:

```go
originalErr := errors.New("connection refused")
wrappedErr := Wrap(originalErr, "failed to connect to database")

fmt.Println(wrappedErr.Error()) // "failed to connect to database: connection refused"
```

### WrapWithContext

Add structured context information:

```go
context := map[string]interface{}{
    "operation": "user_login",
    "user_id":   123,
    "attempt":   3,
}
wrappedErr := WrapWithContext(originalErr, context)

fmt.Println(wrappedErr.Error())
// "operation=user_login user_id=123 attempt=3: connection refused"
```

### JoinErrors

Join multiple errors (Go 1.20+ compatible):

```go
err1 := errors.New("network timeout")
err2 := errors.New("invalid response")
joinedErr := JoinErrors(err1, err2)

// joinedErr contains both errors and can be checked with errors.Is()
```

## Implementation Guidelines

### Error Creation

- **Use factory functions** instead of struct literals for consistency
- **Provide meaningful context** in error messages
- **Include relevant identifiers** (user IDs, file paths, etc.)
- **Chain errors appropriately** using Wrap functions

### Result[T] Usage

- **Use Result[T] for port boundaries** - all service methods should return Result[T]
- **Check result state** before accessing values
- **Prefer Unwrap()** for immediate value extraction when state is known
- **Use Value()/Error()** for safe access patterns

### Error Handling Patterns

```go
// Pattern 1: Immediate return on error
func processData(input string) Result[ProcessedData] {
    validated, err := validateInput(input)
    if err != nil {
        return Err[ProcessedData](Wrap(err, "input validation failed"))
    }

    result, err := process(validated)
    if err != nil {
        return Err[ProcessedData](Wrap(err, "processing failed"))
    }

    return Ok(result)
}

// Pattern 2: Error accumulation
func validateForm(form FormData) Result[ValidatedForm] {
    var errs []error

    if !isValidEmail(form.Email) {
        errs = append(errs, NewValidationError("email", "invalid format", form.Email))
    }

    if len(form.Password) < 8 {
        errs = append(errs, NewValidationError("password", "too short", form.Password))
    }

    if len(errs) > 0 {
        return Err[ValidatedForm](JoinErrors(errs...))
    }

    return Ok(ValidatedForm{Email: form.Email, Password: form.Password})
}
```

## Best Practices

### Error Messages

- **Follow consistent format**: `[ErrorType] context: message`
- **Include actionable information** for debugging
- **Avoid sensitive data** in error messages
- **Use descriptive field names** and identifiers

### Result[T] vs (T, error)

- **Use Result[T]** for all port boundary methods
- **Use (T, error)** only within implementation details
- **Convert between patterns** as needed:

```go
// Convert (T, error) to Result[T]
func toResult[T any](value T, err error) Result[T] {
    if err != nil {
        return Err[T](err)
    }
    return Ok(value)
}

// Convert Result[T] to (T, error)
func fromResult[T any](result Result[T]) (T, error) {
    return result.Unwrap()
}
```

### Error Context

- **Add context at each layer** using Wrap functions
- **Preserve original errors** for `errors.Is()` and `errors.As()` compatibility
- **Use structured context** for machine-readable error information
- **Avoid redundant wrapping** - add value at each level

### Testing

- **Test error creation** and message formatting
- **Test Result[T] patterns** with different types
- **Test error wrapping** preserves original errors
- **Test error interfaces** are properly implemented

## Common Usage Patterns

### Service Layer

```go
type UserService interface {
    CreateUser(email, name string) Result[User]
    GetUser(id string) Result[User]
}

func (s *userService) CreateUser(email, name string) Result[User] {
    // Validate input
    if !isValidEmail(email) {
        return Err[User](NewValidationError("email", "invalid format", email))
    }

    // Check for existing user
    existing, err := s.repo.FindByEmail(email)
    if err != nil {
        return Err[User](Wrap(err, "failed to check existing user"))
    }
    if existing != nil {
        return Err[User](NewValidationError("email", "already exists", email))
    }

    // Create user
    user := User{Email: email, Name: name}
    saved, err := s.repo.Save(user)
    if err != nil {
        return Err[User](Wrap(err, "failed to save user"))
    }

    return Ok(saved)
}
```

### Command Handler

```go
func (h *createUserHandler) Handle(cmd CreateUserCommand) Result[UserCreatedEvent] {
    result := h.userService.CreateUser(cmd.Email, cmd.Name)

    if result.IsErr() {
        // Log error with context
        h.logger.Error("user creation failed",
            "error", result.Error(),
            "email", cmd.Email)

        // Return domain error
        return Err[UserCreatedEvent](result.Error())
    }

    user := result.Value()
    event := UserCreatedEvent{
        UserID: user.ID,
        Email:  user.Email,
        Name:   user.Name,
    }

    return Ok(event)
}
```

### HTTP Adapter

```go
func (h *userHandler) createUser(w http.ResponseWriter, r *http.Request) {
    var cmd CreateUserCommand
    if err := json.NewDecoder(r.Body).Decode(&cmd); err != nil {
        h.respondError(w, NewValidationError("request", "invalid JSON", err.Error()))
        return
    }

    result := h.createUserHandler.Handle(cmd)

    if result.IsErr() {
        h.respondError(w, result.Error())
        return
    }

    event := result.Value()
    h.respondJSON(w, http.StatusCreated, event)
}

func (h *userHandler) respondError(w http.ResponseWriter, err error) {
    var status int

    switch err.(type) {
    case *ValidationError:
        status = http.StatusBadRequest
    case *NotFoundError:
        status = http.StatusNotFound
    default:
        status = http.StatusInternalServerError
    }

    h.respondJSON(w, status, map[string]string{"error": err.Error()})
}
```

This package provides the foundation for consistent, type-safe error handling throughout the Lithos application.
