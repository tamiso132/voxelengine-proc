
use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, NestedMeta, Path, Type};

use crate::FieldConfigs;

pub fn add_field(generated_code: &mut Vec<proc_macro2::TokenStream>, target: &proc_macro2::TokenStream, nested: &proc_macro2::TokenStream, path: &Path, ident: Ident, config: FieldConfigs) {
    let ident_str = ident.to_string();

    if path.is_ident("u8") || path.is_ident("u16") || path.is_ident("u32") || path.is_ident("u64") || path.is_ident("f32") || path.is_ident("f64") || path.is_ident("usize") {
        if !config.slider.0 {
            generated_code.push(quote! {
                let id = ui.push_id(#ident_str);
                ui.text(#ident_str);
                ui.same_line_with_pos(50.0);
                ui.input_scalar("##hidden", &mut #target.#ident).build();
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
                ui.slider("##", #min, #max, &mut #target.#ident);
                id.end();
            });
        }
    } else if path.is_ident("String") {
        generated_code.push(quote! {
            let id = ui.push_id(#ident_str);
            ui.text(#ident_str);
            ui.same_line_with_pos(50.0);
            ui.input_text("##hidden", &mut #target.#ident).build();
            id.end();
        });
    } else if path.is_ident("bool") {
        generated_code.push(quote! {
            let id = ui.push_id(#ident_str);
            ui.text(#ident_str);
            ui.same_line_with_pos(50.0);
            ui.checkbox("##hidden", &mut #target.#ident);
            id.end();
        });
    } else {
        // Must be nested
        generated_code.push(quote! {
            #nested
        });
    }
}