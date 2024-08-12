extern crate proc_macro;

use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, NestedMeta, Path, Type};

#[derive(Default)]
struct FieldConfigs {
    pub slider: (bool, Option<Literal>, Option<Literal>),
}

#[proc_macro_derive(ImGuiFields, attributes(nested, slider))]
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
        let mut config = FieldConfigs::default();

        for attribute in field.attrs.iter() {
            if attribute.path.is_ident("nested") {
                is_nested = true;
                break;
            }

            if attribute.path.is_ident("slider") {
                parse_slider_attribute(&mut config, &attribute);
            }
        }

        match &field.ty {
            Type::Path(type_path) => {
                if is_nested {
                    add_nested(&mut generated_code, &type_path.path, field.ident.clone().unwrap());
                } else {
                    add_field(&mut generated_code, &type_path.path, field.ident.clone().unwrap(), config);
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

fn add_field(generated_code: &mut Vec<proc_macro2::TokenStream>, path: &Path, ident: Ident, config: FieldConfigs) {
    let ident_str = ident.to_string();
    if path.is_ident("u8")
        || path.is_ident("u16")
        || path.is_ident("u32")
        || path.is_ident("u64")
        || path.is_ident("f32")
        || path.is_ident("f64")
        || path.is_ident("usize")
    {
        if !config.slider.0 {
            generated_code.push(quote! {
                let id = ui.push_id(#ident_str);
                ui.text(#ident_str);
                ui.same_line_with_pos(50.0);
                ui.input_scalar("##hidden", &mut self.#ident).build();
                id.end();
            });
        } else {
            if config.slider.1.is_none() || config.slider.2.is_none() {
                panic!("THE SLIDER FIELD HAS NO MIN OR MAX VALUE");
            }
            let min = config.slider.1.unwrap();
            let max = config.slider.2.unwrap();
            generated_code.push(quote! {
                let id = ui.push_id(#ident_str);
                ui.text(#ident_str);
                ui.same_line_with_pos(50.0);
                ui.slider("##", #min, #max, &mut self.#ident);
                id.end();
            });
        }
    } else if path.is_ident("String") {
        generated_code.push(quote! {
            let id = ui.push_id(#ident_str);
            ui.text(#ident_str);
            ui.same_line_with_pos(50.0);
            ui.input_text("##hidden", &mut self.#ident).build();
            id.end();
        });
    } else if path.is_ident("bool") {
        generated_code.push(quote! {
            let id = ui.push_id(#ident_str);
            ui.text(#ident_str);
            ui.same_line_with_pos(50.0);
            ui.checkbox("##hidden", &mut self.#ident);
            id.end();
        });
    }
}

fn add_nested(generated_code: &mut Vec<proc_macro2::TokenStream>, path: &Path, ident: Ident) {
    generated_code.push(quote! {
        self.#ident.render_imgui(&mut ui);
    });
}

fn parse_slider_attribute(config: &mut FieldConfigs, attribute: &Attribute) {
    config.slider.0 = true;

    if let Ok(Meta::List(meta_list)) = attribute.parse_meta() {
        for nested_meta in meta_list.nested {
            match nested_meta {
                NestedMeta::Meta(_) => todo!(),
                NestedMeta::Lit(x) => match x {
                    Lit::Str(_) => todo!(),
                    Lit::ByteStr(_) => todo!(),
                    Lit::Byte(_) => todo!(),
                    Lit::Char(_) => todo!(),
                    Lit::Int(val) => {
                        if config.slider.1.is_none() {
                            config.slider.1 = Some(val.token());
                        } else if config.slider.2.is_none() {
                            config.slider.2 = Some(val.token());
                        }
                    }
                    Lit::Float(val) => {
                        val.token();
                        if config.slider.1.is_none() {
                            config.slider.1 = Some(val.token());
                        } else if config.slider.2.is_none() {
                            config.slider.2 = Some(val.token());
                        }
                    }
                    Lit::Bool(_) => todo!(),
                    Lit::Verbatim(_) => todo!(),
                },
            }
        }
    }
}
