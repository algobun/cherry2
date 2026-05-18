use crate::{FieldDef, ModelDef};
use heck::ToSnakeCase;
use quote::format_ident;
use syn::{Data, DeriveInput, Error, Field, Fields, LitStr};

pub(crate) fn parse_model_def(ast: &DeriveInput) -> Result<ModelDef, Error> {
    let struct_ident = ast.ident.clone();
    let mut table_name = struct_ident.to_string().to_snake_case();

    for attr in &ast.attrs {
        if attr.path().is_ident("orm") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table") {
                    let value: LitStr = meta.value()?.parse()?;
                    table_name = value.value();
                }
                Ok(())
            });
        }
    }

    let data_struct = match &ast.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new(
            struct_ident.span(),
            "Macros can only be derived for structs"
        )),
    };

    let mut fields = Vec::new();
    if let Fields::Named(fields_named) = &data_struct.fields {
        for field in &fields_named.named {
            fields.push(parse_field(field)?);
        }
    }

    Ok(ModelDef {
        crate_name: format_ident!("{}", "cherry"),
        ident: struct_ident,
        table_name,
        fields,
    })
}

fn parse_field(field: &Field) -> Result<FieldDef, Error> {
    let field_ident = field.ident.clone().unwrap();
    let field_type = field.ty.clone();

    let mut is_autoincrement = false;
    let mut is_json = false;
    let mut is_ignored = false;
    let mut column_name = field_ident.to_string().to_snake_case();
    let mut override_column = false;

    for attr in &field.attrs {
        if attr.path().is_ident("orm") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("autoincrement") {
                    is_autoincrement = true;
                } else if meta.path.is_ident("json") {
                    is_json = true;
                } else if meta.path.is_ident("ignore") {
                    is_ignored = true;
                } else if meta.path.is_ident("column") {
                    let value: LitStr = meta.value()?.parse()?;
                    column_name = value.value();
                    override_column = true;
                }
                Ok(())
            });
        }
    }

    
    if is_ignored && (is_autoincrement || is_json || override_column) {
        let msg = "Field marked as 'ignore' cannot also have the following attributes: 'autoincrement', 'json', 'column'.";
        return Err(Error::new(field_ident.span(), msg));
    }

    if is_autoincrement && is_json {
        let msg = "The 'autoincrement' and 'json' attributes cannot be used together.";
        return Err(Error::new(field_ident.span(), msg));
    }
    
    Ok(FieldDef {
        ident: field_ident,
        ty: field_type,
        column_name,
        is_autoincrement,
        is_json,
        is_ignored,
    })
}