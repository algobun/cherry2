extern crate proc_macro;

mod parse;
mod generate;

use crate::parse::parse;
use proc_macro::TokenStream;
// use syn::spanned::Spanned;

/// Internal intermediate representation (IR)
struct ModelDef {
    crate_name: proc_macro2::Ident,
    ident: syn::Ident,
    table_name: String,
    fields: Vec<FieldDef>,
}

struct FieldDef {
    ident: syn::Ident,
    ty: syn::Type,
    column_name: String,
    is_ignored: bool,
    is_autoincrement: bool,
    // is_primary_key: bool,
}

#[proc_macro_derive(Model, attributes(orm))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    parse(input)
}

