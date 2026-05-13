use crate::generate::generate_code;
use crate::{FieldDef, ModelDef};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{parse_macro_input, Data, DeriveInput, Error, Field, Fields, LitStr};

pub(crate) fn parse(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Structural inspection and table name parsing
    let struct_ident = ast.ident.clone();
    // Table default value
    let mut table_name = struct_ident.to_string().to_snake_case();

    // Parse the structure level's #[orm(table = "...")] & override
    for attr in &ast.attrs {
        if attr.path().is_ident("orm") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table") {
                    let value: LitStr = meta.value()?.parse()?;
                    table_name = value.value(); // Override the table name
                }
                Ok(())
            });
        }
    }

    let data_struct = match ast.data {
        Data::Struct(s) => s,
        _ => return Error::new(
                struct_ident.span(), 
                "Model can only be derived for structs"
            )
            .to_compile_error()
            .into(),
    };

    // Field parsing and constraint checking
    let mut fields = Vec::new();
    if let Fields::Named(fields_named) = data_struct.fields {
        for field in fields_named.named {
            match parse_field(field) {
                Ok(def) => fields.push(def),
                Err(e) => return e.to_compile_error().into(),
            }
        }
    }

    let model_def = ModelDef {
        crate_name: format_ident!("{}", "cherry"),
        ident: struct_ident,
        table_name,
        fields,
    };

    generate_code(model_def).into()
}

fn parse_field(field: Field) -> Result<FieldDef, Error> {
    let field_ident = field.ident.unwrap();
    let field_type = field.ty.clone();

    // Fields default values
    let mut is_ignored = false;
    let mut is_autoincrement = false;
    // let mut is_primary_key = false;
    let mut column_name = field_ident.to_string().to_snake_case();

    // Parse field-level attributes & override
    for attr in &field.attrs {
        if attr.path().is_ident("orm") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("ignore") {
                    is_ignored = true;
                } else if meta.path.is_ident("autoincrement") {
                    is_autoincrement = true;
                }
                // else if meta.path.is_ident("primary_key") {
                //     is_primary_key = true;
                // }
                else if meta.path.is_ident("column") {
                    let value: LitStr = meta.value()?.parse()?;
                    column_name = value.value();
                }
                Ok(())
            });
        }
    }

    // [Core Security Gate] Conflict Check
    // if is_ignored && (is_autoincrement || is_primary_key) {
    if is_ignored && is_autoincrement {
        return Err(
            Error::new(
                field_ident.span(),
                "Field cannot be 'ignore' while also being 'autoincrement'" //  or 'primary_key'
            )
        );
    }
    
    Ok(FieldDef {
        ident: field_ident,
        ty: field_type,
        column_name,
        is_autoincrement,
        is_ignored,
        // is_primary_key,
    })
}