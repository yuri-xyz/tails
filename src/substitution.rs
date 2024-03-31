//! A helper module to be used exclusively by the unification module to
//! substitute type variables.

use crate::{assert_extract, symbol_table, types};

#[derive(Debug)]
pub(crate) enum SubstitutionError {
  TypeStripError(types::TypeStripError),
  DirectRecursionCheckError(types::DirectRecursionCheckError),
}

impl From<types::TypeStripError> for SubstitutionError {
  fn from(type_strip_error: types::TypeStripError) -> Self {
    SubstitutionError::TypeStripError(type_strip_error)
  }
}

impl From<types::DirectRecursionCheckError> for SubstitutionError {
  fn from(direct_recursion_check_error: types::DirectRecursionCheckError) -> Self {
    SubstitutionError::DirectRecursionCheckError(direct_recursion_check_error)
  }
}

pub(crate) struct UnificationSubstitutionHelper<'a> {
  pub symbol_table: &'a symbol_table::SymbolTable,
  pub substitution_env: &'a symbol_table::SubstitutionEnv,
}

impl<'a> UnificationSubstitutionHelper<'a> {
  fn substitute_signature_type(
    &self,
    signature_type: &types::SignatureType,
  ) -> Result<types::Type, SubstitutionError> {
    let parameter_types = signature_type
      .parameter_types
      .iter()
      .map(|parameter_type| self.substitute(parameter_type))
      .collect::<Result<Vec<_>, _>>()?;

    let return_type = self.substitute(&signature_type.return_type)?;

    Ok(types::Type::Signature(types::SignatureType {
      parameter_types,
      return_type: Box::new(return_type),
      arity_mode: signature_type.arity_mode,
    }))
  }

  fn substitute_object_type(
    &self,
    object_type: &types::ObjectType,
  ) -> Result<types::Type, SubstitutionError> {
    if let types::ObjectKind::Open(substitution_id) = object_type.kind {
      // SAFETY: What if it wasn't instantiated? Say, it was inside a generic function that was never called? In such a case, this shouldn't fail but the way the instantiation function is built mandates that all types have to be resolved/instantiated. Might need to change that (perhaps by returning an `Option`).
      // SAFETY: Occurs check? Or that doesn't happen here, instead only on unification?

      if let Some(substitution) = self.substitution_env.get(&substitution_id) {
        let substitution_object = assert_extract!(substitution, types::Type::Object);

        // REVIEW: Need to ensure that this logic is correct. If so, add some comments detailing what is happening.
        match substitution_object.kind {
          types::ObjectKind::Open(substitution_substitution_id) => {
            if substitution_substitution_id != substitution_id {
              return self.substitute(substitution);
            }
          }
          types::ObjectKind::Closed => {
            return self.substitute(substitution);
          }
        }
      }
    }

    let substituted_fields = object_type
      .fields
      // OPTIMIZE: Avoid cloning.
      .to_owned()
      .into_iter()
      .map(|(name, field_type)| Ok((name, self.substitute(&field_type)?)))
      .collect::<Result<types::ObjectFieldMap, SubstitutionError>>()?;

    Ok(types::Type::Object(types::ObjectType {
      fields: substituted_fields,
      kind: object_type.kind,
    }))
  }

  /// Substitute a type's entire subtree, substituting any type variable with its
  /// concrete counterpart (if available).
  ///
  /// If the substitution is not defined, the same type is returned. This
  /// function will recursively substitute type variables, until a non-variable
  /// type is encountered.
  ///
  /// In the case that a type variable points to itself (ie. it has no corresponding
  /// monomorphic type in the given substitution environment), the same, unresolved
  /// type variable will be returned. Function callers should account for this.
  pub(crate) fn substitute(&self, ty: &types::Type) -> Result<types::Type, SubstitutionError> {
    // CONSIDER: (test:type_def_nested) On the case that the substitution process ends up on a (nested) polymorphic type stub artifact, it will simply stop its process and return it. This needs to be handle, as it is a hole! Consider improving the substitution function to provide more information about what it did (maybe return an enum alongside the type indicating what was the stopping condition?). Since the type is left with a nested polymorphic stub type, it proceeds to FAIL the concrete assertion!

    // The type should be stripped of all simple, monomorphic stub type
    // layers before processing.
    let stripped_type = ty.to_owned().try_strip_all_stub_layers(self.symbol_table)?;

    match &stripped_type {
      types::Type::Pointer(pointee) => Ok(self.substitute(pointee.as_ref())?.into_pointer_type()),
      types::Type::Object(object_type) => self.substitute_object_type(object_type),
      types::Type::Reference(ty) => Ok(types::Type::Reference(Box::new(
        self.substitute(ty.as_ref())?,
      ))),
      types::Type::Signature(signature_type) => self.substitute_signature_type(signature_type),
      types::Type::Tuple(types::TupleType(element_types)) => {
        Ok(types::Type::Tuple(types::TupleType(
          element_types
            .into_iter()
            .map(|element_type| self.substitute(element_type))
            .collect::<Result<Vec<_>, _>>()?,
        )))
      }
      // In the case that a stub type is encountered after stripping,
      // it must be a polymorphic stub type, which this function cannot handle.
      types::Type::Stub(stub_type) => todo!(),
      types::Type::Variable(types::TypeVariable {
        substitution_id, ..
      }) if self
        .substitution_env
        .get(substitution_id)
        // NOTE: The type doesn't need to be compared by id, since they're both unique
        // per-type, thus it would always be false, which would lead to a stack overflow.
        // Instead, by the point of instantiation it is assumed that both types have been
        // unified, and thus any errors would have been reported.
        .map_or(true, |ty| !ty.is_same_type_variable_as(substitution_id)) =>
      {
        self.substitute(
          self
            .substitution_env
            .get(substitution_id)
            // SAFETY: Undocumented/unchecked unwrap.
            .unwrap(),
        )
      }
      // TODO: Implement. Handle unions.
      types::Type::Union(..) => todo!(),
      // The type is not a stub, generic (at least at this layer), or a fully
      // concrete type. There is nothing to do.
      _ => Ok(ty.to_owned()),
    }
  }
}
