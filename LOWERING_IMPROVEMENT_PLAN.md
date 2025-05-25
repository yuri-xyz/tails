# Lowering Improvement Plan

## Current State Analysis

### Issues Identified
1. **Massive Monolithic File**: 1836 lines in single file with mixed responsibilities
2. **Complex Context Management**: Intricate context passing with mutable state
3. **LLVM API Coupling**: Direct LLVM calls scattered throughout lowering logic
4. **Error Handling Inconsistency**: Mix of panics, unwraps, and proper error handling
5. **Missing Optimization Passes**: No intermediate representation for optimization
6. **Hard to Test**: Tightly coupled to LLVM makes unit testing difficult

### Strengths to Preserve
- Comprehensive language feature coverage
- Proper LLVM integration
- Good separation between different AST node lowering
- Functional approach to code generation

## Improvement Plan (Implementation Order)

### Phase 1: Module Decomposition
**Objective**: Split monolithic lowering into focused modules

**Changes**:
- Create separate modules:
  - `expression_lowering.rs` for expression lowering
  - `statement_lowering.rs` for statement lowering
  - `type_lowering.rs` for type representation
  - `function_lowering.rs` for function compilation
- Extract common utilities to `lowering_utils.rs`
- Maintain shared context through trait bounds

**Benefits**:
- Easier to navigate and maintain
- Clear separation of concerns
- Parallel development possible
- Reduced compilation times

### Phase 2: LLVM Abstraction Layer
**Objective**: Create abstraction over LLVM API for better testability

**Changes**:
- Create `CodeGenerator` trait abstracting LLVM operations
- Implement `LLVMCodeGenerator` for production use
- Add `MockCodeGenerator` for testing
- Define intermediate representation for code generation

**Benefits**:
- Testable lowering logic without LLVM
- Easier to switch backends in future
- Cleaner separation of concerns
- Better error handling

### Phase 3: Context Simplification
**Objective**: Simplify complex context management

**Changes**:
- Create immutable `LoweringContext` with builder pattern
- Separate mutable state into focused managers:
  - `VariableManager` for local variables
  - `FunctionManager` for function state
  - `TypeManager` for type mappings
- Use dependency injection for context components

**Benefits**:
- Clearer ownership and borrowing
- Easier to reason about state changes
- Better error handling
- Reduced complexity

### Phase 4: Error Handling Standardization
**Objective**: Replace panics and unwraps with proper error handling

**Changes**:
- Create `LoweringError` enum with specific error types
- Implement error recovery strategies
- Add error context with source locations
- Replace all panics with proper error returns

**Benefits**:
- Robust compilation pipeline
- Better error messages for users
- Easier debugging of lowering issues
- More reliable compiler

### Phase 5: Intermediate Representation
**Objective**: Add IR layer for optimization and analysis

**Changes**:
- Design simple IR with basic blocks and instructions
- Implement AST to IR lowering
- Add IR to LLVM lowering
- Create IR optimization framework

**Benefits**:
- Platform for optimization passes
- Easier analysis of generated code
- Better debugging of compilation
- Foundation for advanced features

### Phase 6: Testing Infrastructure
**Objective**: Add comprehensive testing for lowering logic

**Changes**:
- Create unit tests for individual lowering functions
- Add integration tests with mock code generator
- Implement golden file testing for IR output
- Add performance benchmarks

**Benefits**:
- Confidence in lowering correctness
- Regression detection
- Performance monitoring
- Development productivity

## Implementation Notes

- Maintain existing functionality during refactoring
- Each phase should be independently valuable
- Consider compilation time impact of abstractions
- Preserve existing LLVM optimization integration
- Add comprehensive tests for each improvement

## Non-Changes (Explicitly Avoided)

- **Custom backend**: LLVM is excellent for code generation
- **Complex IR**: Simple IR is sufficient for current needs
- **Ahead-of-time optimization**: LLVM handles this well
- **Multiple backend support**: Not needed for current goals 