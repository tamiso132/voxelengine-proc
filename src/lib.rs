extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Path, Type};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(nested))]
struct MetaDataStructAttributes {
    nested: String,
}

#[proc_macro_derive(Fields, attributes(nested))]
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
            ui.input_scalar(#ident_str, &mut self.#ident);
        });
    } else if path.is_ident("String") {
        generated_code.push(quote! {
            ui.input_text(#ident_str, &mut self.#ident);
        });
    }
}

fn add_nested(generated_code: &mut Vec<proc_macro2::TokenStream>, path: &Path, ident: Ident) {
    generated_code.push(quote! {
        self.#ident.render_imgui(&mut ui);
    });
}

// fn impl_trait_a(name: &syn::Ident, fields: Fields) -> proc_macro2::TokenStream {
//     quote! {
//         impl Indexable for #name {
//             fn nfields() -> usize {
//                 fields.len();
//             }
//         }
//     }
// }

// fn process_fields(fields: &Fields) -> proc_macro2::TokenStream {
//     let mut field_processors = Vec::new();

//     for field in fields {
//         if let Some(ident) = &field.ident {
//             let field_name = ident.to_string();
//             let field_type = &field.ty;

//             let field_processor = match field_type {
//                 Type::Path(type_path) => {
//                     let path = &type_path.path;
//                     if let Some(last_segment) = path.segments.last() {
//                         // Check if the type is a struct (not a basic type)
//                         if last_segment.ident != "String" && last_segment.ident != "i32" {
//                             let nested_fields = quote! {
//                                 println!("Field: {}, Type: {:?}", #field_name, stringify!(#path));
//                                 // Recursively process nested fields
//                                 // This requires more sophisticated parsing and handling
//                             };
//                             nested_fields
//                         } else {
//                             quote! {
//                                 println!("Field: {}, Type: {:?}", #field_name, stringify!(#path));
//                             }
//                         }
//                     } else {
//                         quote! {
//                             println!("Field: {}, Type: {:?}", #field_name, "Unknown");
//                         }
//                     }
//                 }
//                 _ => quote! {
//                     println!("Field: {}, Type: {:?}", ident, "Unknown");
//                 },
//             };

//             field_processors.push(field_processor);
//         }
//     }

//     quote! {
//         #(#field_processors)*
//     }
// }

// fn test() {}
