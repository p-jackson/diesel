use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::parse::token::str_to_ident;
use syntax::tokenstream::TokenTree;

use attr::Attr;
use util::*;

pub struct Model {
    pub ty: P<ast::Ty>,
    pub attrs: Vec<Attr>,
    pub name: ast::Ident,
    pub generics: ast::Generics,
    pub primary_key_names: Vec<ast::Ident>,
    table_name_from_annotation: Option<ast::Ident>,
}

impl Model {
    pub fn from_annotable(
        cx: &mut ExtCtxt,
        span: Span,
        annotatable: &Annotatable,
    ) -> Option<Self> {
        if let Annotatable::Item(ref item) = *annotatable {
            let table_name_from_annotation =
                str_value_of_attr_with_name(cx, &item.attrs, "table_name");
            let primary_key_names =
                list_value_of_attr_with_name(cx, &item.attrs, "primary_key")
                    .unwrap_or_else(|| vec![str_to_ident("id")]);
            Attr::from_item(cx, item).map(|(generics, attrs)| {
                let ty = struct_ty(cx, span, item.ident, &generics);
                Model {
                    ty: ty,
                    attrs: attrs,
                    name: item.ident,
                    generics: generics,
                    primary_key_names: primary_key_names,
                    table_name_from_annotation: table_name_from_annotation,
                }
            })
        } else {
            None
        }
    }

    pub fn primary_key_names(&self) -> &[ast::Ident] {
        &self.primary_key_names
    }

    pub fn table_name(&self) -> ast::Ident {
        self.table_name_from_annotation.unwrap_or_else(|| {
            str_to_ident(&infer_table_name(&self.name.name.as_str()))
        })
    }

    pub fn attr_named(&self, name: ast::Ident) -> Option<&Attr> {
        self.attrs.iter().find(|attr| {
            attr.column_name.name == name.name
        })
    }

    pub fn field_tokens_for_stable_macro(&self, cx: &mut ExtCtxt) -> Vec<Vec<TokenTree>> {
        self.attrs.iter().map(|a| a.to_stable_macro_tokens(cx)).collect()
    }
}

pub fn infer_association_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    result.push_str(&name[..1].to_lowercase());
    for character in name[1..].chars() {
        if character.is_uppercase() {
            result.push('_');
            for lowercase in character.to_lowercase() {
                result.push(lowercase);
            }
        } else {
            result.push(character);
        }
    }
    result
}

fn infer_table_name(name: &str) -> String {
    let mut result = infer_association_name(name);
    result.push('s');
    result
}

#[test]
fn infer_table_name_pluralizes_and_downcases() {
    assert_eq!("foos", &infer_table_name("Foo"));
    assert_eq!("bars", &infer_table_name("Bar"));
}

#[test]
fn infer_table_name_properly_handles_underscores() {
    assert_eq!("foo_bars", &infer_table_name("FooBar"));
    assert_eq!("foo_bar_bazs", &infer_table_name("FooBarBaz"));
}
