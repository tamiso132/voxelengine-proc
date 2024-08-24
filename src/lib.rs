extern crate proc_macro;
mod bevy_stuff;
use core::panic;

use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Meta, NestedMeta, Path, Type, TypeArray, TypePath};

#[derive(Default, Clone)]
struct FieldConfigs {
    pub slider: (bool, Option<Literal>, Option<Literal>),
}

#[proc_macro_derive(ImGuiFields, attributes(nested, slider, ignore_field))]
pub fn process_fields_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree

    let target = quote! {self};
    let func_ident = quote! {pub fn render_imgui(data: &mut Vec<u8>, ui: *mut imgui::Ui)};

    let item: proc_macro2::TokenStream = input.into();
    let ast: DeriveInput = syn::parse2(item).unwrap();
    let name = &ast.ident;

    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!("Usage of #[Modbus] on a non-struct type"),
    };

    let mut generated_code = Vec::new();

    for field in struct_.fields.iter() {
        let mut config = FieldConfigs::default();
        let mut ignore = false;
        for attribute in field.attrs.iter() {
            if attribute.path.is_ident("slider") {
                parse_slider_attribute(&mut config, &attribute);
            }
            if attribute.path.is_ident("ignore_field") {
                ignore = true;
            }
        }

        if ignore {
            continue;
        }

        let ident = field.ident.clone().unwrap();
        let ident_str = ident.to_string();

        let nested_func = quote! {
            self.#ident.display_nested_imgui(ui, imgui_id, #ident_str);
        };

        match &field.ty {
            Type::Path(type_path) => {
                
                add_field(&mut generated_code, &target, &nested_func, &type_path.path, field.ident.clone().unwrap(), config);
            }
          

            _ => {
                panic!("Only Path is implemented");
            }
        }
    }
    let expanded = quote! {
            impl TImguiRender for #name {
              fn display_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId) {
                        #(#generated_code)*
                }

                fn display_nested_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId, ident: &str){

                    ui.text(ident);

                    #(#generated_code)*
                }
            }
    };

    proc_macro::TokenStream::from(expanded)
}

// Type::Array(array_type) => {
//     panic!("WTF");
//     // Assuming `array_type` is of type `TypeArray`
//     if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = &array_type.len {
//         let len: usize = lit_int.base10_parse().expect("Expected integer literal for array length");

//         if let Type::Path(TypePath { path, .. }) = &*array_type.elem {
//             for i in 0..len {
//                 let s = format!("{}[{}]", ident_str, i);
//                 let ident = Ident::new(&s, proc_macro2::Span::call_site());
//                 panic!("Ident: {}", ident.to_string());
//               //  add_field(&mut generated_code, &target, &nested_func, &path, ident, config.clone());
//             }
//         }
//     } else {
//         panic!("Array length is not a literal integer");
//     }
// }

#[proc_macro_derive(BevyField, attributes(slider))]
pub fn process_fields_derive_bevy(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: proc_macro2::TokenStream = input.into();
    let mut ast: DeriveInput = syn::parse2(item).unwrap();
    let name = &ast.ident;

    let target = quote! {value};

    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!("Usage of #[Modbus] on a non-struct type"),
    };

    let mut generated_code = Vec::new();

    for field in struct_.fields.iter() {
        let mut config = FieldConfigs::default();

        for attribute in field.attrs.iter() {
            if attribute.path.is_ident("slider") {
                parse_slider_attribute(&mut config, &attribute);
            }
        }

        let ident = field.ident.clone().unwrap();

        let ident_ptr = Ident::new(&format!("{}_ptr", ident), ident.span());
        let ident_vec = Ident::new(&format!("{}_vec", ident), ident.span());

        let nested = quote! {
            // let #ident_ptr = ((&mut value.#ident) as *mut #ident).cast::<u8>();
            // let #ident_vec = std::slice::from_raw_parts_mut(v, std::mem::size_of::<#ident>());
            // #ident::display_imgui(value, ui);
        };

        match &field.ty {
            Type::Path(type_path) => {
                bevy_stuff::add_field(&mut generated_code, &target, &nested, &type_path.path, field.ident.clone().unwrap(), config);
            }
            _ => {
                todo!();
            }
        }
    }
    let expanded = quote! {
        impl TReflect for  #name {


            fn display_imgui(data: &mut [u8], ui: *mut imgui::Ui) {
            unsafe{

            let ptr = align_ptr(data.as_mut_ptr(), align_of::<#name>()).cast::<#name>();
            let value = &mut *ptr;
            let ui = &mut *ui;

            #(#generated_code)*
            }
        }
        }

        impl #name{
            pub fn register(type_registry: &AppTypeRegistry){
                let b = type_registry.write().add_registration(#name::get_type_registration());
                type_registry.write().register_type_data::<#name, ReflectTypeData>();
            }
        }

    };

    proc_macro::TokenStream::from(expanded)
}

fn add_field(generated_code: &mut Vec<proc_macro2::TokenStream>, target: &proc_macro2::TokenStream, nested: &proc_macro2::TokenStream, path: &Path, ident: Ident, config: FieldConfigs) {
    let ident_str = ident.to_string();

    if path.is_ident("u8") || path.is_ident("u16") || path.is_ident("u32") || path.is_ident("u64") || path.is_ident("f32") || path.is_ident("f64") || path.is_ident("usize") {
        if !config.slider.0 {
            generated_code.push(quote! {
                voxelengine_gui::display::display_scalar(ui, #ident_str, imgui_id, &mut #target.#ident);
            });
        } else {
            if config.slider.1.is_none() || config.slider.2.is_none() {
                panic!("THE SLIDER FIELD HAS NO MIN OR MAX VALUE");
            }
            let min = config.slider.1.unwrap();
            let max = config.slider.2.unwrap();
            generated_code.push(quote! {
                 voxelengine_gui::display::display_slider(ui, #ident_str, imgui_id, #min, #max, &mut #target.#ident);
            });
        }
    } else if path.is_ident("String") {
        generated_code.push(quote! {
            voxelengine_gui::display::display_text(ui, #ident_str, imgui_id, &mut #target.#ident);
        });
    } else if path.is_ident("bool") {
        generated_code.push(quote! {
             voxelengine_gui::display::display_boolean(ui, #ident_str, imgui_id, &mut #target.#ident);
        });
    } else {
        generated_code.push(quote! {
            #nested
        });
    }
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
