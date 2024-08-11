extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Path, Type};

#[proc_macro_derive(ImGuiFields, attributes(nested))]
pub fn process_fields_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree

    let item: proc_macro2::TokenStream = input.into();
    let mut ast: DeriveInput = syn::parse2(item).unwrap();
    let name = &ast.ident;

    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!("Usage of #[Modbus] on a non-struct type"),
    };

    let mut generated_code = Vec::new();

    for field in struct_.fields.iter() {
        let mut is_nested = false;
        for attribute in field.attrs.iter() {
            if attribute.path.is_ident("nested") {
                is_nested = true;
                break;
            }
        }

        match &field.ty {
            Type::Path(type_path) => {
                if is_nested {
                    add_nested(&mut generated_code, &type_path.path, field.ident.clone().unwrap());
                } else {
                    add_field(&mut generated_code, &type_path.path, field.ident.clone().unwrap());
                }
            }
            _ => {
                todo!();
            }
        }
    }
    let expanded = quote! {
        impl #name {
            pub fn render_imgui(&mut self, mut ui: &mut imgui::Ui) {
                #(#generated_code)*
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_field(generated_code: &mut Vec<proc_macro2::TokenStream>, path: &Path, ident: Ident) {
    let ident_str = ident.to_string();
    if path.is_ident("u8") || path.is_ident("u16") || path.is_ident("u32") || path.is_ident("u64") || path.is_ident("f32") || path.is_ident("f64") {
        generated_code.push(quote! {
            ui.input_scalar(#ident_str, &mut self.#ident).build();
        });
    } else if path.is_ident("String") {
        generated_code.push(quote! {
            ui.input_text(#ident_str, &mut self.#ident).build();
        });
    } else if path.is_ident("bool") {
        generated_code.push(quote! {
            ui.checkbox(#ident_str, &mut self.#ident);
        });
    }
}

fn add_nested(generated_code: &mut Vec<proc_macro2::TokenStream>, path: &Path, ident: Ident) {
    generated_code.push(quote! {
        self.#ident.render_imgui(&mut ui);
    });
}
