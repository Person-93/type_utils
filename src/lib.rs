use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::ToTokens;
use std::fmt::{Display, Formatter};
use syn::{
  parse_macro_input, Attribute, FieldsNamed, FieldsUnnamed, Generics, Variant, Visibility,
};

mod parse_impl;
mod to_tokens_impl;

#[proc_macro_derive(TypeUtils, attributes(tu_derive, pick, omit))]
pub fn type_utils(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as TypeUtils)
    .into_token_stream()
    .into()
}

#[derive(Clone)]
struct TypeUtils {
  kind: TypeKind,
  attrs: Vec<Attribute>,
  actions: Vec<Action>,
  ident: Ident,
}

#[derive(Clone)]
struct Action {
  kind: ActionKind,
  derives: Vec<Ident>,
  vis: Visibility,
  ident: Ident,
  generics: Generics,
  selection: Selection,
}

#[derive(Copy, Clone)]
enum ActionKind {
  Pick,
  Omit,
}

impl Display for ActionKind {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ActionKind::Pick => write!(f, "pick"),
      ActionKind::Omit => write!(f, "omit"),
    }
  }
}

#[derive(Clone)]
struct Selection(Vec<SelectedField>);

#[derive(Clone)]
struct SelectedField {
  vis: Visibility,
  ident: Ident,
}

#[derive(Clone)]
enum TypeKind {
  Struct(FieldsNamed),
  TupleStruct(FieldsUnnamed),
  UnitStruct,
  Enum(Vec<Variant>),
}

impl TypeKind {
  fn display(&self) -> &'static str {
    match self {
      TypeKind::Struct(_) => "struct",
      TypeKind::TupleStruct(_) => "tuple struct",
      TypeKind::UnitStruct => "unit struct",
      TypeKind::Enum(_) => "enum",
    }
  }
}
