# Parser Phase Improvement Plan

## Current State Analysis

### Issues Identified
1. **Monolithic Parser Struct**: Single large struct handling all parsing responsibilities
2. **Manual Precedence Management**: Hand-coded precedence climbing with magic numbers
3. **Inconsistent Error Recovery**: Some methods recover from errors, others don't
4. **Token Stream Management**: Manual index tracking with bounds checking scattered throughout
5. **Mixed Responsibilities**: Parser handles both syntax analysis and some semantic validation
6. **Large Methods**: Some parsing methods exceed 100 lines with complex nested logic

### Strengths to Preserve
- Comprehensive syntax coverage
- Good test coverage for core functionality
- Clear separation between expressions and statements
- Proper handling of operator precedence

## Improvement Plan (Implementation Order)

### Phase 1: Token Stream Abstraction
**Objective**: Replace manual token index management with proper abstraction

**Changes**:
- Create `TokenStream` struct wrapping token vector
- Implement `peek()`, `next()`, `expect()`, `is_at_end()` methods
- Add `checkpoint()` and `restore()` for backtracking
- Remove all manual index manipulation from parser

**Benefits**:
- Eliminates index-out-of-bounds errors
- Cleaner error messages with position context
- Easier to implement lookahead when needed

### Phase 2: Error Recovery Strategy
**Objective**: Implement consistent error recovery across all parsing methods

**Changes**:
- Define `ParseError` enum with recovery strategies
- Implement synchronization points (statement boundaries, block ends)
- Add `try_parse()` methods that don't consume tokens on failure
- Create error recovery helpers for common patterns

**Benefits**:
- Better error messages for users
- Parser continues after errors to find more issues
- More robust parsing of malformed input

### Phase 3: Precedence Table Abstraction
**Objective**: Replace hardcoded precedence with configurable table

**Changes**:
- Create `PrecedenceTable` struct with operator definitions
- Define associativity rules alongside precedence
- Implement generic precedence climbing algorithm
- Move precedence values to constants with descriptive names

**Benefits**:
- Easier to modify operator precedence
- Self-documenting precedence relationships
- Reduces magic numbers in code

### Phase 4: Parser Decomposition
**Objective**: Split monolithic parser into focused sub-parsers

**Changes**:
- Create `ExpressionParser`, `StatementParser`, `TypeParser` structs
- Each sub-parser handles its domain with shared `TokenStream`
- Implement `ParseContext` for shared state (symbol tables, etc.)
- Main parser coordinates between sub-parsers

**Benefits**:
- Single responsibility principle
- Easier to test individual parsing components
- Clearer code organization

### Phase 5: AST Builder Pattern
**Objective**: Separate AST construction from parsing logic

**Changes**:
- Create `AstBuilder` trait with methods for each AST node type
- Implement position tracking and metadata attachment in builder
- Add validation logic to builder methods
- Parser uses builder instead of direct AST construction

**Benefits**:
- Centralized AST node creation
- Consistent metadata attachment
- Easier to add AST transformations

### Phase 6: Syntax Validation Separation
**Objective**: Move semantic validation out of parser

**Changes**:
- Remove type checking from parser methods
- Focus parser on pure syntax analysis
- Create separate validation pass for semantic checks
- Parser only validates syntactic correctness

**Benefits**:
- Cleaner separation of concerns
- Parser becomes more focused and maintainable
- Easier to modify semantic rules independently

## Implementation Notes

- Maintain existing test suite during refactoring
- Each phase should be independently testable
- Consider performance impact of additional abstractions
- Keep parser methods focused on single constructs
- Preserve existing error message quality

## Non-Changes (Explicitly Avoided)

- **Generated parsers**: Hand-written parser provides better error messages
- **Left-recursive grammar**: Current approach handles precedence well
- **Backtracking parser**: Would complicate error reporting
- **Separate lexer integration**: Current tight coupling is beneficial 