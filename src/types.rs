//! Defines all the various types of the type system, and type-related utility
//! functions and structs which are used during the various type system phases
//! such as constraint gathering, unification or instantiation.

use crate::{
  ast,
  symbol_table::{self, SubstitutionEnv},
};

/// Object fields must be an ordered map (such as a binary tree map),
/// because otherwise their positions would be non-deterministic. This
/// would cause consistency problems of emitted LLVM IR, such as when
/// comparing codegen tests.
pub type ObjectFieldMap = std::collections::BTreeMap<String, Type>;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ObjectKind {
  /// The object is open and can be extended.
  ///
  /// Represents a fragment, or a part of an overall object type.
  Open(symbol_table::SubstitutionId),
  /// The object is closed and cannot be extended. Usually used
  /// for object literal's types.
  ///
  /// Represents a complete object type.
  Closed,
}

#[derive(Clone, Debug)]
pub struct ObjectType {
  pub fields: ObjectFieldMap,
  /// Describes the kind of object type. Used to aid with type inference
  /// of objects during type unification.
  pub kind: ObjectKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArityMode {
  Variadic {
    /// Used to allow variadic foreign functions to specify the minimum amount
    /// of fixed parameters that are required during signature type unification.
    minimum_required_parameters: usize,
  },
  /// The signature is not variadic, and its parameter count is always a fixed
  /// amount.
  Fixed,
}

impl ArityMode {
  pub fn is_variadic(&self) -> bool {
    matches!(self, ArityMode::Variadic { .. })
  }

  pub fn get_minimum_required_parameters(&self) -> Option<usize> {
    match self {
      ArityMode::Variadic {
        minimum_required_parameters,
      } => Some(*minimum_required_parameters),
      _ => None,
    }
  }
}

#[derive(Clone, Debug)]
pub struct SignatureType {
  pub return_type: Box<Type>,
  pub parameter_types: Vec<Type>,
  pub arity_mode: ArityMode,
}

/// Represents a type that needs to be resolved.
///
/// Type stubs are ultimately resolved to types that may be declared, for example
/// type definitions and generics. However, type stubs can point to other type stubs
/// because just the reference to a type declaration is considered a type stub.
///
/// Type stubs can only point to: type definitions, generics, and unions.
#[derive(Debug, Clone)]
pub struct StubType {
  pub path: ast::Path,
}

impl StubType {
  /// Remove all non-polymorphic stub layers to simplify a stub type.
  ///
  /// This operation is shallow, and will not affect any inner types. Also,
  /// if a type stub layer with generic hints is encountered, that type stub
  /// will be returned (it will not be processed from there on). This is because
  /// in that case, instantiation logic would be required.
  ///
  /// If recursive types (via stub types) are encountered, the function will fail
  /// with the corresponding error variant. However, if recursive types exist but
  /// they are nested, the function will succeed because of separation of concerns;
  /// this function is only concerned with stripping all stub type layers.
  pub(crate) fn strip_all_stub_layers(
    self,
    symbol_table: &symbol_table::SymbolTable,
  ) -> Result<Type, TypeStripError> {
    // TODO: Use this function as part of the `substitute` method. To be able to do this, make this function part of `Type`'s implementation.
    // OPTIMIZE: Use reference to avoid taking ownership of `self`.

    let mut current = self;

    // Strip away all stub layers that have no generic parameters; they
    // are simple layers and don't require any kind of instantiation or
    // substitution.
    loop {
      let target_registry_item = symbol_table
        .follow_link(&current.path.link_id)
        .ok_or(TypeStripError::SymbolTableMissingEntry)?;

      let next = match target_registry_item {
        // TODO: Handle unions case.
        symbol_table::RegistryItem::Union(union) => todo!(),
        symbol_table::RegistryItem::TypeDef(type_def) => type_def.body.to_owned(),
        _ => unreachable!("all possible stub type targets should have been covered"),
      };

      if let Type::Stub(next_stub_type) = next {
        current = next_stub_type;
      } else {
        return Ok(next);
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct TupleType(pub Vec<Type>);

#[derive(PartialEq, PartialOrd, Copy, Clone, Debug, Eq)]
pub enum BitWidth {
  Width8 = 8,
  Width16 = 16,
  Width32 = 32,
  Width64 = 64,
  // TODO: Add support for 128-bit size.
  Width128 = 128,
}

#[derive(PartialEq, Clone, Debug, Eq)]
pub enum PrimitiveType {
  /// An integer literal with its bit size, and whether it is
  /// signed.
  Integer(BitWidth, bool),
  Real(BitWidth),
  Bool,
  Char,
  CString,
}

#[derive(Clone, Debug)]
pub struct TypeVariable {
  pub substitution_id: symbol_table::SubstitutionId,
  pub debug_name: &'static str,
}

impl TypeVariable {
  pub fn try_substitute_self<'a>(&'a self, substitution_env: &'a SubstitutionEnv) -> Option<&Type> {
    substitution_env.get(&self.substitution_id).and_then(|ty| {
      if !ty.is_same_type_variable_as(&self.substitution_id) {
        Some(ty)
      } else {
        None
      }
    })
  }

  pub fn has_substitution(&self, substitution_env: &SubstitutionEnv) -> bool {
    substitution_env
      .get(&self.substitution_id)
      .map_or(false, |substitution| {
        !substitution.is_same_type_variable_as(&self.substitution_id)
      })
  }
}

pub struct ImmediateSubtreeIterator<'a> {
  stack: Vec<Box<dyn Iterator<Item = &'a Type> + 'a>>,
}

impl<'a> ImmediateSubtreeIterator<'a> {
  pub fn new(root: &'a Type) -> Self {
    Self {
      stack: vec![root.get_inner_types()],
    }
  }
}

impl<'a> Iterator for ImmediateSubtreeIterator<'a> {
  type Item = &'a Type;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(mut branch) = self.stack.pop() {
      if let Some(ty) = branch.next() {
        // Insert the remaining items on the stack.
        self.stack.push(branch);

        self.stack.push(ty.get_inner_types());

        return Some(ty);
      }
    }

    None
  }
}

/// Traverse the indirect subtree of a given type, resolving type stubs as they
/// are encountered.
///
/// This function calls the provided visitor on the resolved targets of any type
/// stubs, rather than the stubs themselves. As a result, type stubs within the
/// subtree are never directly visited; instead, the function explores their
/// associated target types.
pub(crate) struct IndirectSubtreeIterator<'a> {
  stack: Vec<Type>,
  symbol_table: &'a symbol_table::SymbolTable,
}

impl<'a> Iterator for IndirectSubtreeIterator<'a> {
  type Item = Result<Type, TypeStripError>;

  fn next(&mut self) -> Option<Self::Item> {
    let ty = match self.stack.pop() {
      Some(ty) => ty,
      None => return None,
    };

    let stripped_type = match ty.try_strip_all_stub_layers(self.symbol_table) {
      Ok(stripped_type) => stripped_type,
      Err(type_strip_error) => return Some(Err(type_strip_error)),
    };

    self
      .stack
      // OPTIMIZE: Avoid cloning.
      .extend(stripped_type.get_inner_types().cloned().collect::<Vec<_>>());

    Some(Ok(stripped_type))
  }
}

#[derive(Debug)]
pub(crate) enum TypeStripError {
  SymbolTableMissingEntry,
}

#[derive(Debug)]
pub(crate) enum DirectRecursionCheckError {
  SymbolTableMissingEntry,
}

pub enum Type2<T> {
  Primitive(PrimitiveType),
  Pointer(Box<T>),
  Reference(Box<T>),
  Tuple(TupleType),
  Object(ObjectType),
  Signature(SignatureType),
  Union(ast::Union),
  Range(u64, u64),
  Opaque,
  Unit,
}

pub struct ConcreteType(Type2<ConcreteType>);

pub enum PreUnificationType {
  Type(Type2<PreUnificationType>),
  Variable(TypeVariable),
  Stub(StubType),
}

#[derive(Clone, Debug)]
pub enum Type {
  Union(std::rc::Rc<ast::Union>),
  Range(u64, u64),
  Primitive(PrimitiveType),
  Pointer(Box<Type>),
  /// An opaque pointer. Equivalent to a pointer to void (void*) or to an unknown
  /// type.
  Opaque,
  Reference(Box<Type>),
  Tuple(TupleType),
  Object(ObjectType),
  Stub(StubType),
  Signature(SignatureType),
  /// A meta type to be used during unification.
  ///
  /// Represents a type that has not yet been solved. A type variable will
  /// take the form of a monomorphic (ground) type once unification has
  /// been performed.
  Variable(TypeVariable),
  /// A meta type that represents the lack of a value.
  Unit,
}

impl Type {
  pub(crate) fn get_immediate_subtree_iter(&self) -> ImmediateSubtreeIterator<'_> {
    ImmediateSubtreeIterator::new(self)
  }

  pub(crate) fn try_strip_all_stub_layers(
    self,
    symbol_table: &symbol_table::SymbolTable,
  ) -> Result<Type, TypeStripError> {
    // OPTIMIZE: Use `std::borrow::Cow`.

    if let Type::Stub(stub_type) = self {
      stub_type.strip_all_stub_layers(symbol_table)
    } else {
      Ok(self)
    }
  }

  pub(crate) fn into_pointer_type(self) -> Type {
    Type::Pointer(Box::new(self))
  }

  pub(crate) fn is_same_type_variable_as(&self, id: &symbol_table::SubstitutionId) -> bool {
    if let Type::Variable(TypeVariable {
      substitution_id, ..
    }) = self
    {
      return substitution_id == id;
    }

    false
  }

  /// Determine whether the type is the unit type.
  ///
  /// This determination will not perform flattening.
  pub(crate) fn is_a_unit(&self) -> bool {
    matches!(self, Type::Unit)
  }

  pub fn is_a_meta(&self) -> bool {
    matches!(self, Type::Stub(..) | Type::Variable(..))
  }

  /// A concrete type is any type that is not a meta type (ex. stub,
  /// ype variable, etc.) and whose entire inner type subtree is also
  /// concrete.
  pub(crate) fn is_immediate_subtree_concrete(&self) -> bool {
    // NOTE: Nested stub types without generic hints (non-polymorphic stub types)
    // might seem like they may be considered concrete (because they would simply
    // be simple stub layers), but they shouldn't be actually considered concrete.
    // This is because that same stub type could resolve to a non-concrete type, such
    // as a generic. Instead, this function's purpose focuses to ensure that a given
    // type is FULLY concrete and simplified.
    !self.is_a_meta() && self.get_immediate_subtree_iter().all(|ty| !ty.is_a_meta())
  }

  pub(crate) fn get_inner_types(&self) -> Box<dyn Iterator<Item = &Type> + '_> {
    match self {
      Type::Pointer(pointee) => Box::new(std::iter::once(pointee.as_ref())),
      Type::Object(object) => Box::new(object.fields.iter().map(|field| field.1)),
      Type::Tuple(TupleType(element_types)) => Box::new(element_types.iter()),
      Type::Reference(pointee) => Box::new(std::iter::once(pointee.as_ref())),
      Type::Signature(signature) => Box::new(signature.parameter_types.iter()),
      // TODO: Handle unions case.
      Type::Union(union_) => todo!(),
      _ => Box::new(std::iter::empty()),
    }
  }

  // CONSIDER: Add a `find_substitution_id` helper function (or trait) that will perform abstract operations on substitute-able types, such as type variables and `typeof` types. For example, it would re-perform the unification operation with its substitution if it is bound, and also perform occurs checks. This would standardize the process of substitution.
}
