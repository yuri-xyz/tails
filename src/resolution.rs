use crate::{
  symbol_table,
  types::{self, TypeStripError},
};

pub(crate) type UniverseStack = Vec<symbol_table::UniverseId>;

#[derive(Debug)]
pub(crate) enum TypeResolutionError {
  StubTypeMissingSymbolTableEntry,
  EmptyUniverseStackWhenResolvingGeneric,
  CouldNotFindSubstitutionInAnyUniverseInUniverseStack,
  NoUniversesWhenResolvingGeneric,
}

impl From<types::DirectRecursionCheckError> for TypeResolutionError {
  fn from(error: types::DirectRecursionCheckError) -> Self {
    match error {
      types::DirectRecursionCheckError::SymbolTableMissingEntry => {
        TypeResolutionError::StubTypeMissingSymbolTableEntry
      }
    }
  }
}

impl From<types::DirectRecursionCheckError> for TypeStripError {
  fn from(error: types::DirectRecursionCheckError) -> Self {
    match error {
      types::DirectRecursionCheckError::SymbolTableMissingEntry => {
        TypeStripError::SymbolTableMissingEntry
      }
    }
  }
}

#[derive(Debug)]
pub(crate) enum TypeResolutionByIdError {
  MissingEntryForTypeId,
  TypeResolutionError(TypeResolutionError),
}

pub(crate) fn push_to_universe_stack(
  mut universe_stack: UniverseStack,
  new_universe_id: symbol_table::UniverseId,
) -> Result<UniverseStack, &'static str> {
  if universe_stack.contains(&new_universe_id) {
    return Err("universe stack should not contain the new universe id already");
  }

  universe_stack.push(new_universe_id);

  Ok(universe_stack)
}

pub(crate) struct ResolutionHelper<'a> {
  pub base: BaseResolutionHelper<'a>,
  pub type_env: &'a symbol_table::TypeEnvironment,
}

impl<'a> ResolutionHelper<'a> {
  pub fn new(
    symbol_table: &'a symbol_table::SymbolTable,
    type_env: &'a symbol_table::TypeEnvironment,
  ) -> Self {
    let base = BaseResolutionHelper::new(symbol_table);

    Self { base, type_env }
  }

  pub(crate) fn resolve_by_id(
    &'a self,
    type_id: &symbol_table::TypeId,
    universe_stack: UniverseStack,
  ) -> Result<std::borrow::Cow<'a, types::Type>, TypeResolutionByIdError> {
    let ty = self
      .type_env
      .get(type_id)
      .ok_or(TypeResolutionByIdError::MissingEntryForTypeId)?;

    self
      .base
      .resolve(ty, universe_stack)
      .map_err(|type_resolution_error| {
        TypeResolutionByIdError::TypeResolutionError(type_resolution_error)
      })
  }
}

pub(crate) struct BaseResolutionHelper<'a> {
  symbol_table: &'a symbol_table::SymbolTable,
}

impl<'a> BaseResolutionHelper<'a> {
  pub fn new(symbol_table: &'a symbol_table::SymbolTable) -> Self {
    Self { symbol_table }
  }

  /// Recursively instantiate stub type if applicable, then substitute it with
  /// a monomorphic type if it is a generic/polymorphic type.
  ///
  /// This will return a concrete type, with a concrete immediate subtree.
  pub(crate) fn resolve<'b>(
    &'b self,
    ty: &'b types::Type,
    universe_stack: UniverseStack,
  ) -> Result<std::borrow::Cow<'b, types::Type>, TypeResolutionError> {
    // Nothing to do if the type is already fully concrete.
    if ty.is_immediate_subtree_concrete() {
      return Ok(std::borrow::Cow::Borrowed(ty));
    }

    let resolution = match ty {
      types::Type::Stub(stub_type) => self.resolve_stub_type(stub_type, universe_stack)?,
      // The type is not a stub, generic (at least at this layer), or a fully concrete type.
      // In other words, the type contains a nested stub, or generic at some level on its
      // subtree.
      _ => self.resolve_within_subtree(ty, universe_stack)?,
    };

    assert!(
      resolution.is_immediate_subtree_concrete(),
      "resolved type should be concrete"
    );

    Ok(resolution)
  }

  fn resolve_within_subtree<'b>(
    &self,
    ty: &types::Type,
    universe_stack: UniverseStack,
  ) -> Result<std::borrow::Cow<'b, types::Type>, TypeResolutionError> {
    Ok(std::borrow::Cow::Owned(match ty {
      types::Type::Pointer(pointee) => types::Type::Pointer(Box::new(
        self.resolve(pointee, universe_stack)?.into_owned(),
      )),
      types::Type::Reference(pointee) => types::Type::Reference(Box::new(
        self.resolve(pointee, universe_stack)?.into_owned(),
      )),
      types::Type::Tuple(tuple) => types::Type::Tuple(types::TupleType(
        tuple
          .0
          .iter()
          // FIXME: Properly handle result.
          // OPTIMIZE: Avoid cloning.
          .map(|ty| self.resolve(ty, universe_stack.clone()))
          .collect::<Result<Vec<_>, _>>()?
          .into_iter()
          .map(|cow| cow.into_owned())
          .collect(),
      )),
      types::Type::Object(object_type) => {
        let fields = object_type.fields.iter().try_fold(
          std::collections::BTreeMap::new(),
          |mut accumulator, field| -> Result<_, TypeResolutionError> {
            accumulator.insert(
              field.0.to_owned(),
              // OPTIMIZE: Avoid cloning.
              self.resolve(field.1, universe_stack.clone())?.into_owned(),
            );

            Ok(accumulator)
          },
        )?;

        types::Type::Object(types::ObjectType {
          fields,
          kind: object_type.kind,
        })
      }
      types::Type::Signature(signature) => {
        let return_type = self
          .resolve(&signature.return_type, universe_stack.clone())?
          .into_owned();

        let parameter_types = signature
          .parameter_types
          .iter()
          // OPTIMIZE: Avoid cloning.
          .map(|param_type| self.resolve(param_type, universe_stack.clone()))
          .collect::<Result<Vec<_>, _>>()?
          .into_iter()
          .map(|cow| cow.into_owned())
          .collect();

        types::Type::Signature(types::SignatureType {
          arity_mode: signature.arity_mode,
          parameter_types,
          return_type: Box::new(return_type),
        })
      }
      _ => unreachable!(
        "type should have been a type constructor by this point, with a nested generic or stub type"
      ),
    }))
  }

  pub(crate) fn resolve_stub_type<'b>(
    &'b self,
    stub_type: &'b types::StubType,
    universe_stack: UniverseStack,
  ) -> Result<std::borrow::Cow<'b, types::Type>, TypeResolutionError> {
    let stripped_target = stub_type
      // OPTIMIZE: Avoid cloning.
      .clone()
      .strip_all_monomorphic_stub_layers(self.symbol_table)
      .or(Err(TypeResolutionError::StubTypeMissingSymbolTableEntry))?;

    dbg!(stripped_target.clone());

    let resolved_target = self.resolve(&stripped_target, universe_stack)?;

    // OPTIMIZE: Avoid cloning; currently only cloning to satisfy borrow checker.
    return Ok(std::borrow::Cow::Owned(resolved_target.into_owned()));
  }
}
