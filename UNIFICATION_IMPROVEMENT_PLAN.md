# Unification Improvement Plan

## Current State Analysis

### Issues Identified
1. **Monolithic Context**: `TypeUnificationContext` handles too many responsibilities
2. **Manual Occurs Check**: Hand-rolled infinite type detection with potential bugs
3. **Scattered Substitution Logic**: Type substitution spread across multiple methods
4. **Poor Error Context**: Generic unification failures without specific reasons
5. **Object Type Complexity**: Special-cased object unification with complex logic
6. **Missing Constraint Types**: Only equality constraints supported

### Strengths to Preserve
- Solid foundation of Hindley-Milner algorithm
- Proper handling of type variables and substitutions
- Good separation between constraint generation and solving
- Comprehensive test coverage for basic cases

## Improvement Plan (Implementation Order)

### Phase 1: Constraint System Expansion
**Objective**: Support more constraint types beyond equality

**Changes**:
- Extend `Constraint` enum with:
  - `Subtype(Type, Type)` for variance
  - `HasField(Type, String, Type)` for object access
  - `TupleElement(Type, usize, Type)` for tuple indexing
- Implement constraint-specific unification logic
- Add constraint simplification rules

**Benefits**:
- More expressive type system
- Better error messages for specific failures
- Foundation for advanced type features

### Phase 2: Substitution Engine Refactor
**Objective**: Centralize and optimize type substitution

**Changes**:
- Create `SubstitutionEngine` struct with:
  - `apply(ty: &Type, env: &SubstitutionEnv) -> Type`
  - `compose(env1: &SubstitutionEnv, env2: &SubstitutionEnv) -> SubstitutionEnv`
  - `occurs_check(var: SubstitutionId, ty: &Type) -> bool`
- Implement efficient substitution with memoization
- Add substitution validation and debugging

**Benefits**:
- Centralized substitution logic
- Performance optimizations through caching
- Easier to debug substitution issues

### Phase 3: Unification Algorithm Decomposition
**Objective**: Split unification into focused, testable components

**Changes**:
- Create specialized unifiers:
  - `PrimitiveUnifier` for basic types
  - `StructuralUnifier` for compound types
  - `VariableUnifier` for type variables
  - `ObjectUnifier` for object types
- Implement `Unifier` trait with common interface
- Coordinate unifiers through main context

**Benefits**:
- Single responsibility principle
- Easier to test individual unification cases
- Cleaner code organization

### Phase 4: Error Context Enhancement
**Objective**: Provide detailed context for unification failures

**Changes**:
- Create `UnificationError` enum with specific variants:
  - `TypeMismatch { expected, actual, context }`
  - `OccursCheck { variable, type_expr }`
  - `ArityMismatch { expected, actual }`
  - `FieldMismatch { field, expected, actual }`
- Add error context tracking through unification
- Implement error message formatting with type pretty-printing

**Benefits**:
- Much better error messages for users
- Easier debugging of type issues
- More helpful IDE integration

### Phase 5: Occurs Check Optimization
**Objective**: Implement efficient and correct occurs checking

**Changes**:
- Use union-find data structure for variable equivalence
- Implement path compression for variable chains
- Add cycle detection with proper error reporting
- Cache occurs check results for performance

**Benefits**:
- Correct handling of infinite types
- Better performance for complex type expressions
- Clearer error messages for cyclic types

### Phase 6: Object Unification Simplification
**Objective**: Simplify complex object type unification logic

**Changes**:
- Implement row polymorphism for object types
- Separate open/closed object handling
- Add field presence/absence constraints
- Simplify object substitution logic

**Benefits**:
- Cleaner object type semantics
- More predictable object unification
- Foundation for structural typing

## Implementation Notes

- Maintain existing test suite during refactoring
- Each phase should improve performance or correctness
- Consider memory usage of additional data structures
- Preserve deterministic unification behavior
- Add comprehensive benchmarks

## Non-Changes (Explicitly Avoided)

- **Higher-ranked types**: Too complex for current language goals
- **Type classes/traits**: Not in language specification
- **Dependent types**: Would complicate unification significantly
- **Linear types**: Not needed for current use cases 