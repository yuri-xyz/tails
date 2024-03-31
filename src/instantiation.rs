//! Responsible for instantiating generic and polymorphic types, and associating
//! their artifacts with the corresponding substitution tables "universes".
//!
//! In other words, this module is responsible for the "instantiation" phase of the compiler,
//! which mainly consists of the creation of "universes", which are then used on later
//! phases of the compiler to resolve generics, polymorphic types, and other artifacts.

use crate::{ast, inference, resolution, symbol_table, types, unification};

pub(crate) type ReverseUniverseTracker =
  std::collections::HashMap<symbol_table::RegistryId, Vec<symbol_table::UniverseId>>;

/// Contains substitution environments for generic types.
pub(crate) type TypeSchemes =
  std::collections::HashMap<symbol_table::UniverseId, symbol_table::SubstitutionEnv>;

#[derive(Debug, Clone)]
pub enum Artifact {
  CallSite(std::rc::Rc<ast::CallSite>),
  StubType(types::StubType),
}

pub(crate) struct InstantiationHelper<'a> {
  pub universes: TypeSchemes,
  symbol_table: &'a symbol_table::SymbolTable,
}

impl<'a> InstantiationHelper<'a> {
  /// Unify two types for equality to determine whether they are
  /// equal.
  pub fn compare_by_unification(
    type_a: types::Type,
    type_b: types::Type,
    symbol_table: &symbol_table::SymbolTable,
  ) -> bool {
    // TODO: Get rid of this. Type comparisons should only occur during the unification process.

    let universes = TypeSchemes::new();

    let mut type_unification_context =
      unification::TypeUnificationContext::new(symbol_table, symbol_table::SubstitutionEnv::new());

    let constraints = vec![inference::Constraint::Equality(type_a, type_b)]
      .into_iter()
      .map(|constraint| (resolution::UniverseStack::new(), constraint))
      .collect();

    type_unification_context
      .solve_constraints(&symbol_table::TypeEnvironment::new(), &constraints)
      .is_ok()
  }

  pub(crate) fn new(symbol_table: &'a symbol_table::SymbolTable) -> Self {
    Self {
      universes: TypeSchemes::new(),
      symbol_table,
    }
  }
}
