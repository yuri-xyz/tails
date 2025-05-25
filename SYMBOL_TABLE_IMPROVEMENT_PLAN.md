# Symbol Table Improvement Plan

## Current State Analysis

### Issues Identified
1. **Fragmented Storage**: Multiple separate hash maps for different concerns
2. **Weak Type Safety**: Raw `usize` IDs without type-level guarantees
3. **Manual Link Management**: Error-prone manual link following with unwraps
4. **Missing Metadata**: No source location or scope information stored
5. **Inconsistent Access Patterns**: Some direct registry access, some through links
6. **No Scope Hierarchy**: Flat structure doesn't represent lexical scoping

### Strengths to Preserve
- Clear separation between registry and links
- Efficient hash-based lookups
- Support for forward references
- Clean abstraction for different symbol kinds

## Improvement Plan (Implementation Order)

### Phase 1: Type-Safe ID System
**Objective**: Replace raw `usize` IDs with type-safe wrappers

**Changes**:
- Create newtype wrappers: `RegistryId(NonZeroUsize)`, `LinkId(NonZeroUsize)`
- Use `NonZeroUsize` to enable `Option<ID>` optimizations
- Add `From` implementations for ergonomic conversions
- Implement `Display` for debugging

**Benefits**:
- Prevents ID type confusion at compile time
- Memory optimization for optional IDs
- Better debugging output

### Phase 2: Unified Symbol Entry
**Objective**: Consolidate symbol information into single entry type

**Changes**:
- Create `SymbolEntry` struct containing:
  - `item: RegistryItem`
  - `location: SourceLocation`
  - `scope_id: ScopeId`
  - `visibility: Visibility`
  - `flags: SymbolFlags`
- Replace separate hash maps with single `entries: HashMap<RegistryId, SymbolEntry>`
- Add helper methods for common access patterns

**Benefits**:
- Single source of truth for symbol information
- Easier to add new metadata fields
- Reduced memory fragmentation

### Phase 3: Scope Hierarchy
**Objective**: Implement proper lexical scoping with parent-child relationships

**Changes**:
- Create `Scope` struct with `parent: Option<ScopeId>` and `children: Vec<ScopeId>`
- Add `scopes: HashMap<ScopeId, Scope>` to symbol table
- Implement scope resolution walking up parent chain
- Add scope creation/destruction methods

**Benefits**:
- Proper lexical scoping semantics
- Enables shadowing detection
- Foundation for closure capture analysis

### Phase 4: Safe Link Resolution
**Objective**: Replace unwrap-heavy link following with safe alternatives

**Changes**:
- Create `LinkResolver` trait with methods:
  - `resolve(&self, link_id: LinkId) -> Option<&SymbolEntry>`
  - `resolve_mut(&mut self, link_id: LinkId) -> Option<&mut SymbolEntry>`
  - `resolve_chain(&self, link_id: LinkId) -> Vec<&SymbolEntry>`
- Implement error handling for broken links
- Add link validation methods

**Benefits**:
- Eliminates runtime panics from broken links
- Better error messages for resolution failures
- Easier debugging of symbol resolution

### Phase 5: Query Interface
**Objective**: Provide high-level query interface for common operations

**Changes**:
- Create `SymbolQuery` builder pattern:
  - `by_name(name: &str) -> SymbolQuery`
  - `in_scope(scope_id: ScopeId) -> SymbolQuery`
  - `of_kind(kind: SymbolKind) -> SymbolQuery`
  - `visible_from(scope_id: ScopeId) -> SymbolQuery`
- Implement iterator-based results
- Add caching for expensive queries

**Benefits**:
- Declarative symbol lookup
- Consistent visibility rules
- Performance optimization opportunities

### Phase 6: Source Location Integration
**Objective**: Track source locations for all symbols

**Changes**:
- Add `SourceLocation` struct with file, line, column
- Store location in `SymbolEntry`
- Implement location-based queries
- Add location to error messages

**Benefits**:
- Better error reporting with source context
- Enables IDE features like go-to-definition
- Debugging assistance

## Implementation Notes

- Maintain backward compatibility during transition
- Add comprehensive tests for each improvement
- Consider memory usage impact of additional metadata
- Preserve existing performance characteristics
- Keep API simple and focused

## Non-Changes (Explicitly Avoided)

- **Database-backed storage**: In-memory hash maps are sufficient
- **Persistent symbol tables**: Not needed for single compilation
- **Complex query language**: Simple builder pattern is adequate
- **Automatic garbage collection**: Manual management is more predictable 