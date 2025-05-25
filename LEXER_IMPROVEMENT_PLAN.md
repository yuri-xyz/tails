# Lexer Phase Improvement Plan

## Current State Analysis

### Issues Identified
1. **Manual Character Processing**: Hand-rolled character iteration with index management
2. **Inconsistent Error Handling**: Mix of `Option` and `Result` types without clear strategy
3. **Hardcoded Token Mapping**: Large match statement for keyword identification
4. **Stateful Indentation Logic**: Complex indent/dedent tracking with mutable state
5. **Limited Unicode Support**: ASCII-only with TODO comments about unicode handling
6. **Scattered Validation**: Character validation spread across multiple methods

### Strengths to Preserve
- Clear separation of concerns between lexing and parsing
- Comprehensive test coverage
- Proper indentation handling for Python-like syntax
- Good token position tracking

## Improvement Plan (Implementation Order)

### Phase 1: Error Handling Standardization
**Objective**: Replace inconsistent error handling with unified approach

**Changes**:
- Create `LexerError` enum with specific error variants
- Replace all `Option` returns with `Result<T, LexerError>`
- Add position information to all errors
- Implement proper error recovery strategies

**Benefits**:
- Consistent error reporting
- Better debugging information
- Cleaner API surface

### Phase 2: Character Stream Abstraction
**Objective**: Replace manual index management with iterator-based approach

**Changes**:
- Create `CharStream` struct wrapping `Peekable<Enumerate<Chars>>`
- Implement `peek()`, `next()`, `position()` methods
- Remove manual index tracking from `Lexer`
- Add `take_while()` and `skip_while()` convenience methods

**Benefits**:
- Eliminates index-out-of-bounds bugs
- Cleaner character consumption logic
- Better abstraction boundaries

### Phase 3: Token Factory Pattern
**Objective**: Centralize token creation and validation

**Changes**:
- Create `TokenFactory` with methods like `make_identifier()`, `make_number()`
- Move keyword matching to factory
- Add token validation at creation time
- Implement token position calculation in factory

**Benefits**:
- Single responsibility for token creation
- Easier to add new token types
- Centralized validation logic

### Phase 4: Indentation State Machine
**Objective**: Simplify indentation tracking with explicit state machine

**Changes**:
- Create `IndentationTracker` struct
- Define clear states: `LineStart`, `Indenting`, `Content`, `Dedenting`
- Implement state transitions with clear rules
- Separate indentation logic from main lexer

**Benefits**:
- Clearer indentation semantics
- Easier to debug indentation issues
- More maintainable code

### Phase 5: Number Parsing Improvements
**Objective**: Robust numeric literal handling

**Changes**:
- Support scientific notation (1e10, 1.5e-3)
- Add binary (0b), octal (0o), hex (0x) literals
- Implement proper overflow detection
- Add underscore separators (1_000_000)

**Benefits**:
- More expressive numeric literals
- Better error messages for invalid numbers
- Consistent with modern language standards

## Implementation Notes

- Each phase can be implemented independently
- Maintain backward compatibility during transitions
- Add comprehensive tests for each improvement
- Consider performance impact of abstractions
- Keep lexer stateless where possible

## Non-Changes (Explicitly Avoided)

- **Regex-based lexing**: Would complicate error reporting and position tracking
- **Lookahead beyond 1 character**: Current design is sufficient
- **Complex macro system**: Would violate language simplicity principles
- **Automatic semicolon insertion**: Not needed for Python-like syntax 