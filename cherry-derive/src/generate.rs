use crate::ModelDef;
use quote::{format_ident, quote};


pub(crate) fn generate_code(model: ModelDef) -> proc_macro2::TokenStream {
    let crate_name = &model.crate_name;
    let struct_ident = &model.ident;
    let table_name = &model.table_name;

    let mut const_columns = Vec::new();
    let mut meta_columns = Vec::new();
    let mut insert_values = Vec::new();
    let mut row_reads = Vec::new();

    for field in model.fields {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let col_name = &field.column_name;
        let is_ignore = field.is_ignored;
        // let is_pk = field.is_primary_key;
        let is_auto = field.is_autoincrement;

        // Handle the ignored fields
        if field.is_ignored {
            // 被忽略的字段在从数据库读取时，使用 Rust 的 Default 特性填充
            row_reads.push(quote! {
                #field_ident: ::std::default::Default::default()
            });
            continue;
        }

        // Generate uppercase Column constants (for DSL)
        let const_name = format_ident!("{}", field_ident.to_string().to_uppercase());
        const_columns.push(quote! {
            pub const #const_name: ::#crate_name::query::dsl::Column<#field_ty> =
                ::#crate_name::query::dsl::Column::new(#col_name);
        });

        // Generate ColumnMeta array elements (for framework metadata)
        meta_columns.push(quote! {
            ::#crate_name::model::ColumnMeta {
                name: #col_name,
                is_ignore: #is_ignore,
                // is_primary_key: #is_pk,
                is_autoincrement: #is_auto,
            }
        });

        // Generate insert_values logic (compile-time filtered out self-increment column)
        if !field.is_autoincrement {
            insert_values.push(quote! {
                (#col_name, ::#crate_name::DbType::to_db_value(&self.#field_ident))
            });
        }

        // Generate the logic for mapping from Row
        row_reads.push(quote! {
            #field_ident: {
                let db_val = ::#crate_name::DbType::from_db_value(
                    row.get(#col_name)
                       .map_err(|e| ::#crate_name::Error::ParseError(e.to_string()))?
                )?;
                db_val
            }
        });
    }

    // 最终组装代码
    quote! {
        // 常量关联块
        impl #struct_ident {
            #(#const_columns)*
        }

        // Trait 实现块
        impl ::#crate_name::model::Model for #struct_ident {
            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [::#crate_name::model::ColumnMeta] {
                &[ #(#meta_columns),* ]
            }

            fn to_values(&self) -> ::std::vec::Vec<(&'static str, ::#crate_name::DbValue)> {
                ::std::vec![ #(#insert_values),* ]
            }

            fn from_row(row: &::#crate_name::Row) -> ::#crate_name::Result<Self> {
                ::#crate_name::Result::Ok(Self {
                    #(#row_reads),*
                })
            }
        }
    }
}
