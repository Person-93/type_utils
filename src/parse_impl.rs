use crate::{Action, ActionKind, SelectedField, Selection, TypeKind, TypeUtils};
use nullable_result::{extract, IterExt, MaybeTryFrom, MaybeTryInto, NullableResult};
use proc_macro2::{Ident, Span};
use syn::{
  braced, parenthesized,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Generics, Result, Token,
  Visibility,
};

impl Parse for TypeUtils {
  fn parse(input: ParseStream) -> Result<Self> {
    let input: DeriveInput = input.parse()?;
    input.try_into()
  }
}

impl TryFrom<DeriveInput> for TypeUtils {
  type Error = Error;

  fn try_from(input: DeriveInput) -> Result<Self> {
    let utils = TypeUtils {
      kind: input.data.try_into()?,
      actions: {
        let actions = input
          .attrs
          .into_iter()
          .map(MaybeTryInto::maybe_try_into)
          .filter_nulls()
          .collect::<Result<Vec<_>>>()?;
        if actions.is_empty() {
          return Err(Error::new(
            Span::call_site(),
            "TypeUtils derived with no actions",
          ));
        }
        actions
      },
      ident: input.ident,
    };

    type Validator<'v> = Box<dyn FnMut((ActionKind, &Ident, &SelectedField)) -> Result<()> + 'v>;

    let validate: Validator = match &utils.kind {
      TypeKind::Struct(target_fields) => Box::new(|(action_kind, _action_ident, action_field)| {
        if target_fields
          .named
          .iter()
          .map(|named| named.ident.as_ref().unwrap())
          .any(|target_ident| target_ident == &action_field.ident)
        {
          Ok(())
        } else {
          Err(Error::new(
            action_field.ident.span(),
            format!(
              "unknown field `{}` in {action_kind} action for {}",
              action_field.ident, utils.ident
            ),
          ))
        }
      }),
      TypeKind::TupleStruct(_) => Box::new(|(action_kind, _action_ident, _action_field)| {
        Err(Error::new(
          Span::call_site(),
          format!("{action_kind} action is not supported for tuple structs"),
        ))
      }),
      TypeKind::UnitStruct => Box::new(|(action_kind, _action_ident, _action_field)| {
        Err(Error::new(
          Span::call_site(),
          format!("{action_kind} action is not supported for unit structs"),
        ))
      }),
      TypeKind::Enum(variants) => Box::new(|(action_kind, _action_ident, action_field)| {
        if variants
          .iter()
          .any(|variant| variant.ident == action_field.ident)
        {
          Ok(())
        } else {
          Err(Error::new(
            action_field.ident.span(),
            format!(
              "unknown variant `{}` in {action_kind} action for {}",
              action_field.ident, utils.ident
            ),
          ))
        }
      }),
    };

    utils
      .actions
      .iter()
      .map(|action| {
        action
          .selection
          .0
          .iter()
          .map(|field| (action.kind, &action.ident, field))
      })
      .flatten()
      .try_for_each(validate)?;

    Ok(utils)
  }
}

impl TryFrom<Data> for TypeKind {
  type Error = Error;

  fn try_from(data: Data) -> Result<Self> {
    match data {
      Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
      }) => Ok(TypeKind::Struct(fields)),
      Data::Struct(DataStruct {
        fields: Fields::Unnamed(fields),
        ..
      }) => Ok(TypeKind::TupleStruct(fields)),
      Data::Struct(DataStruct {
        fields: Fields::Unit,
        ..
      }) => Ok(TypeKind::UnitStruct),
      Data::Enum(DataEnum { variants, .. }) => Ok(TypeKind::Enum(
        variants.into_iter().map(From::from).collect(),
      )),
      Data::Union(_) => Err(Error::new(
        Span::call_site(),
        "TypeUtils cannot be used on union types",
      )),
    }
  }
}

impl MaybeTryFrom<Attribute> for Action {
  type Error = Error;

  fn maybe_try_from(attr: Attribute) -> NullableResult<Self, Self::Error> {
    let kind = match attr.path.get_ident() {
      Some(ident) if ident == "pick" => ActionKind::Pick,
      Some(ident) if ident == "omit" => ActionKind::Omit,
      Some(_) | None => return NullableResult::Null,
    };

    let Partial {
      vis,
      ident,
      generics,
      selection,
    } = extract!(syn::parse2::<Partial>(attr.tokens));

    return NullableResult::Ok(Action {
      kind,
      vis,
      ident,
      generics,
      selection,
    });

    struct Partial {
      vis: Visibility,
      ident: Ident,
      generics: Generics,
      selection: Selection,
    }

    impl Parse for Partial {
      fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(Partial {
          vis: content.parse()?,
          ident: content.parse()?,
          generics: {
            let mut generics: Generics = content.parse()?;
            generics.where_clause = content.parse()?;
            generics
          },
          selection: content.parse()?,
        })
      }
    }
  }
}

impl Parse for Selection {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    braced!(content in input);
    Ok(Self(
      Punctuated::<_, Token![,]>::parse_separated_nonempty(&content)?
        .into_iter()
        .collect::<Vec<_>>(),
    ))
  }
}

impl Parse for SelectedField {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(SelectedField {
      vis: input.parse()?,
      ident: input.parse()?,
    })
  }
}
