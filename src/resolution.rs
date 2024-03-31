use crate::{
  symbol_table,
  types::{self, TypeStripError},
};

#[derive(Debug)]
pub(crate) enum TypeResolutionError {
  StubTypeMissingSymbolTableEntry,
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
  ) -> Result<std::borrow::Cow<'a, types::Type>, TypeResolutionByIdError> {
    let ty = self
      .type_env
      .get(type_id)
      .ok_or(TypeResolutionByIdError::MissingEntryForTypeId)?;

    self.base.resolve(ty).map_err(|type_resolution_error| {
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
  ) -> Result<std::borrow::Cow<'b, types::Type>, TypeResolutionError> {
    // Nothing to do if the type is already fully concrete.
    if ty.is_immediate_subtree_concrete() {
      return Ok(std::borrow::Cow::Borrowed(ty));
    }

    let resolution = match ty {
      types::Type::Stub(stub_type) => self.resolve_stub_type(stub_type)?,
      // The type is not a stub, or a fully concrete type. In other words,
      // the type contains a nested stub  at some level on its subtree.
      _ => self.resolve_within_subtree(ty)?,
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
  ) -> Result<std::borrow::Cow<'b, types::Type>, TypeResolutionError> {
    Ok(std::borrow::Cow::Owned(match ty {
      types::Type::Pointer(pointee) => {
        types::Type::Pointer(Box::new(self.resolve(pointee)?.into_owned()))
      }
      types::Type::Reference(pointee) => {
        types::Type::Reference(Box::new(self.resolve(pointee)?.into_owned()))
      }
      types::Type::Tuple(tuple) => types::Type::Tuple(types::TupleType(
        tuple
          .0
          .iter()
          // FIXME: Properly handle result.
          // OPTIMIZE: Avoid cloning.
          .map(|ty| self.resolve(ty))
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
              self.resolve(field.1)?.into_owned(),
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
        let return_type = self.resolve(&signature.return_type)?.into_owned();

        let parameter_types = signature
          .parameter_types
          .iter()
          // OPTIMIZE: Avoid cloning.
          .map(|param_type| self.resolve(param_type))
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
  ) -> Result<std::borrow::Cow<'b, types::Type>, TypeResolutionError> {
    let stripped_target = stub_type
      // OPTIMIZE: Avoid cloning.
      .clone()
      .strip_all_stub_layers(self.symbol_table)
      .or(Err(TypeResolutionError::StubTypeMissingSymbolTableEntry))?;

    let resolved_target = self.resolve(&stripped_target)?;

    // OPTIMIZE: Avoid cloning; currently only cloning to satisfy borrow checker.
    return Ok(std::borrow::Cow::Owned(resolved_target.into_owned()));
  }
}
