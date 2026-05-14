extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod generate;
mod parse;

// Internally shared data structures (IR)

pub(crate) struct ModelDef {
    pub crate_name: syn::Ident,
    pub ident: syn::Ident,
    pub table_name: String,
    pub fields: Vec<FieldDef>,
}

pub(crate) struct FieldDef {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub column_name: String,
    pub is_autoincrement: bool,
    pub is_ignored: bool,
}

#[proc_macro_derive(Model, attributes(orm))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    // Invoke the shared parsing logic
    let model_def = match parse::parse_model_def(&ast) {
        Ok(def) => def,
        Err(e) => return e.to_compile_error().into(),
    };

    // For Model, generate both Model implementation and FromRow implementation.
    let from_row_code = generate::generate_from_row_code(&model_def);
    let model_code = generate::generate_model_code(&model_def);

    let expanded = quote::quote! {
        #from_row_code
        #model_code
    };

    expanded.into()
}

#[proc_macro_derive(FromRow, attributes(orm))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    // Invoke the shared parsing logic
    let model_def = match parse::parse_model_def(&ast) {
        Ok(def) => def,
        Err(e) => return e.to_compile_error().into(),
    };

    // Only generate the implementation of FromRow
    generate::generate_from_row_code(&model_def).into()
}
