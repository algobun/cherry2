use crate::ModelDef;
use quote::{format_ident, quote};

/// Dedicated to generating `Model` core functionality and DSL constants (exclusive to `Model` only)
pub(crate) fn generate_model_code(model: &ModelDef) -> proc_macro2::TokenStream {
    let crate_name = &model.crate_name;
    let struct_ident = &model.ident;
    let table_name = &model.table_name;

    let mut const_columns = Vec::new();
    // let mut meta_columns = Vec::new();
    let mut insert_values = Vec::new();

    for field in &model.fields {
        if field.is_ignored {
            continue;
        }

        let field_ident = &field.ident;
        let field_type = &field.ty;
        let column_name = &field.column_name;
        let is_auto = field.is_autoincrement;
        let is_json = field.is_json;
        let is_ignore = field.is_ignored;

        let column_generic_type = if is_json {
            // Wrap `Json` Type for JSON field
            quote! { ::#crate_name::types::json::Json<#field_type> }
        } else {
            quote! { #field_type }
        };


        // Generate associated constants for columns
        let const_name = format_ident!("{}", field_ident.to_string());
        const_columns.push(quote! {
            #[allow(non_upper_case_globals)]
            pub const #const_name: ::#crate_name::query::dsl::Column<#column_generic_type> =
                ::#crate_name::query::dsl::Column::new(
                    ::#crate_name::types::ColumnMeta {
                        table_name: #table_name,
                        column_name: #column_name,
                        is_autoincrement: #is_auto,
                        is_json: #is_json,
                        is_ignore: #is_ignore,
                    }
                );
        });

        // Filter the auto-increment columns' insert_values
        if !is_auto {
            let value_logic = if is_json {
                quote! { ::#crate_name::types::json::to_db(&self.#field_ident)? }
            } else {
                quote! { ::#crate_name::types::Type::to_db(&self.#field_ident)? }
            };
            
            insert_values.push(quote! { (#column_name, #value_logic) });
        }
    }

    quote! {
        impl #struct_ident {
            #(#const_columns)*
        }

        impl ::#crate_name::types::Model for #struct_ident {
            fn table_name() -> &'static str {
                #table_name
            }

            fn to_values(&self) -> ::#crate_name::Result<::std::vec::Vec<(&'static str, ::#crate_name::types::Value)>> {
                ::#crate_name::Result::Ok( ::std::vec![ #(#insert_values),* ] )
            }
        }
    }
}

/// Dedicated for generating `FromRow` code (common to `Model` and `DTO`)
pub(crate) fn generate_from_row_code(model: &ModelDef) -> proc_macro2::TokenStream {
    let crate_name = &model.crate_name;
    let struct_ident = &model.ident;
    let table_name = &model.table_name;

    let mut meta_columns = Vec::new();
    let mut row_reads = Vec::new();

    for field in &model.fields {
        let field_ident = &field.ident;
        // let field_ty = &field.ty;
        let col_name = &field.column_name;
        let is_auto = field.is_autoincrement;
        let is_json = field.is_json;
        let is_ignore = field.is_ignored;

        // Handle the ignored fields
        if field.is_ignored {
            row_reads.push(quote! {
                #field_ident: ::std::default::Default::default()
            });
            continue;
        }

        meta_columns.push(quote! {
            ::#crate_name::types::ColumnMeta {
                table_name: #table_name,
                column_name: #col_name,
                is_autoincrement: #is_auto,
                is_json: #is_json,
                is_ignore: #is_ignore,
            }
        });

        let strict_col_name = format!("{}.{}", table_name, col_name);
        let parse_logic = if is_json {
            quote! { ::#crate_name::types::json::from_db(val)? }
        } else {
            quote! { ::#crate_name::types::Type::from_db(val)? }
        };

        row_reads.push(quote! {
            #field_ident: {
                let val: #crate_name::types::Value =
                    // Prioritize attempts to get strict column names with table name prefixes
                    // `table_name.column_name`
                    row.get(#strict_col_name).or_else(|_|
                        // Smart degradation to pure column name lookup if strict column name does not exist
                        // (for native `AS` alias or pure `DTO` scenarios)
                        row.get(#col_name).map_err(|e| ::#crate_name::Error::ParseError(e.to_string()))
                )?;
                # parse_logic
            }
        });
    }

    // Only generating `FromRow` implementation
    quote! {
        impl ::#crate_name::types::FromRow for #struct_ident {

            fn columns() -> std::borrow::Cow<'static, [::#crate_name::types::ColumnMeta]> {
                std::borrow::Cow::Borrowed( &[ #(#meta_columns),* ] )
            }

            fn from_row(row: &::#crate_name::types::Row) -> ::#crate_name::Result<Self> {
                ::#crate_name::Result::Ok(Self {
                    #(#row_reads),*
                })
            }
        }
    }
}
