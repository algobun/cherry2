use crate::ModelDef;
use quote::{format_ident, quote};

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
        let is_ignore = field.is_ignored;

        // Handle the ignored fields
        if field.is_ignored {
            row_reads.push(quote! {
                #field_ident: ::std::default::Default::default()
            });
            continue;
        }

        meta_columns.push(quote! {
            ::#crate_name::model::ColumnMeta {
                table_name: #table_name,
                column_name: #col_name,
                is_ignore: #is_ignore,
                is_autoincrement: #is_auto,
            }
        });


        let strict_col_name = format!("{}.{}", table_name, col_name);

        row_reads.push(quote! {
            #field_ident: {
                // Prioritize attempts to get strict column names with table name prefixes
                // `table_name.column_name`
                let val_res: ::std::result::Result<::#crate_name::Value, _> = row.get(#strict_col_name);

                // Smart degradation to pure column name lookup if strict column name does not exist
                // (for native `AS` alias or pure `DTO` scenarios)
                let db_val = match val_res {
                    ::std::result::Result::Ok(val) => val,
                    ::std::result::Result::Err(_) => {
                        row.get(#col_name).map_err(|e| ::#crate_name::Error::ParseError(e.to_string()))?
                    }
                };

                ::#crate_name::Type::from_db(db_val)?
            }
        });
    }

    // Only generating `FromRow` implementation
    quote! {
        impl ::#crate_name::model::FromRow for #struct_ident {

            fn columns() -> std::borrow::Cow<'static, [::#crate_name::model::ColumnMeta]> {
                std::borrow::Cow::Borrowed( &[ #(#meta_columns),* ] )
            }

            fn from_row(row: &::#crate_name::model::Row) -> ::#crate_name::Result<Self> {
                ::#crate_name::Result::Ok(Self {
                    #(#row_reads),*
                })
            }
        }
    }
}

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
        let field_ty = &field.ty;
        let column_name = &field.column_name;
        let is_auto = field.is_autoincrement;
        let is_ignore = field.is_ignored;

        // Generate associated constants for columns
        let const_name = format_ident!("{}", field_ident.to_string());
        const_columns.push(quote! {
            #[allow(non_upper_case_globals)]
            pub const #const_name: ::#crate_name::query::dsl::Column<#field_ty> =
                ::#crate_name::query::dsl::Column::new(
                    ::#crate_name::model::ColumnMeta {
                        table_name: #table_name,
                        column_name: #column_name,
                        is_ignore: #is_ignore,
                        is_autoincrement: #is_auto,
                    }
                );
        });

        // Filter the auto-increment columns' insert_values
        if !is_auto {
            insert_values.push(quote! {
                (#column_name, ::#crate_name::Type::to_db(&self.#field_ident)?)
            });
        }
    }

    quote! {
        impl #struct_ident {
            #(#const_columns)*
        }

        impl ::#crate_name::model::Model for #struct_ident {
            fn table_name() -> &'static str {
                #table_name
            }

            fn to_values(&self) -> ::#crate_name::Result<::std::vec::Vec<(&'static str, ::#crate_name::Value)>> {
                ::#crate_name::Result::Ok( ::std::vec![ #(#insert_values),* ] )
            }
        }
    }
}