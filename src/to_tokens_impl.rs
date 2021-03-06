use crate::{Action, ActionKind, TypeKind, TypeUtils};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
  AttrStyle, Attribute, Fields, FieldsNamed, Item, ItemEnum, ItemStruct, Path, Visibility,
};

impl ToTokens for TypeUtils {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend((*self).clone().into_token_stream());
  }

  fn into_token_stream(self) -> TokenStream
  where
    Self: Sized,
  {
    let mut tokens = TokenStream::new();
    for action in self.actions {
      action
        .into_item(&self.kind, &self.attrs)
        .to_tokens(&mut tokens);
    }
    tokens
  }
}

impl Action {
  fn into_item(self, type_kind: &TypeKind, attrs: &[Attribute]) -> Item {
    match (self.kind, type_kind) {
      (ActionKind::Pick, TypeKind::Struct(fields)) => Item::Struct(ItemStruct {
        attrs: {
          let mut attrs = attrs.to_vec();
          if let Some(derive) = expand_derives(&self.derives) {
            attrs.insert(0, derive);
          }
          attrs
        },
        vis: self.vis,
        struct_token: Default::default(),
        ident: self.ident,
        generics: self.generics,
        fields: Fields::Named(FieldsNamed {
          brace_token: Default::default(),
          named: fields
            .named
            .iter()
            .filter_map(|field| {
              self
                .selection
                .0
                .iter()
                .find(|selected_field| &selected_field.ident == field.ident.as_ref().unwrap())
                .map(|selected| {
                  let mut field = field.clone();
                  field.vis = match &selected.vis {
                    Visibility::Inherited => field.vis,
                    vis => vis.clone(),
                  };
                  field
                })
            })
            .collect(),
        }),
        semi_token: None,
      }),
      (ActionKind::Pick, TypeKind::Enum(variants)) => Item::Enum(ItemEnum {
        attrs: {
          let mut attrs = attrs.to_vec();
          if let Some(derive) = expand_derives(&self.derives) {
            attrs.insert(0, derive);
          }
          attrs
        },
        vis: self.vis,
        enum_token: Default::default(),
        ident: self.ident,
        generics: self.generics,
        brace_token: Default::default(),
        variants: variants
          .iter()
          .filter_map(|variant| {
            self
              .selection
              .0
              .iter()
              .find(|selected| selected.ident == variant.ident)
              .map(|_| variant.clone())
          })
          .collect(),
      }),
      (ActionKind::Omit, TypeKind::Struct(fields)) => Item::Struct(ItemStruct {
        attrs: {
          let mut attrs = attrs.to_vec();
          if let Some(derive) = expand_derives(&self.derives) {
            attrs.insert(0, derive);
          }
          attrs
        },
        vis: self.vis,
        struct_token: Default::default(),
        ident: self.ident,
        generics: self.generics,
        fields: Fields::Named(FieldsNamed {
          brace_token: Default::default(),
          named: fields
            .named
            .iter()
            .filter_map(|field| {
              self
                .selection
                .0
                .iter()
                .all(|selected_field| &selected_field.ident != field.ident.as_ref().unwrap())
                .then(|| field.clone())
            })
            .collect(),
        }),
        semi_token: None,
      }),
      (ActionKind::Omit, TypeKind::Enum(variants)) => Item::Enum(ItemEnum {
        attrs: {
          let mut attrs = attrs.to_vec();
          if let Some(derive) = expand_derives(&self.derives) {
            attrs.insert(0, derive);
          }
          attrs
        },
        vis: self.vis,
        enum_token: Default::default(),
        ident: self.ident,
        generics: self.generics,
        brace_token: Default::default(),
        variants: variants
          .iter()
          .filter_map(|variant| {
            self
              .selection
              .0
              .iter()
              .all(|selected| selected.ident != variant.ident)
              .then(|| variant.clone())
          })
          .collect(),
      }),
      (action_kind, type_kind) => {
        panic!("{action_kind} action selected for {}", type_kind.display())
      }
    }
  }
}

fn expand_derives(derives: &[Ident]) -> Option<Attribute> {
  if derives.is_empty() {
    None
  } else {
    Some(Attribute {
      pound_token: Default::default(),
      style: AttrStyle::Outer,
      bracket_token: Default::default(),
      path: Path::from(format_ident!("derive")),
      tokens: quote! { (#(#derives),*) },
    })
  }
}
