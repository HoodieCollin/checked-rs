use convert_case::{Case, Casing};
use proc_macro_error::abort;
use quote::format_ident;
use syn::parse_quote;

use super::attr_params::AttrParams;

pub struct StructItem {
    pub vis: syn::Visibility,
    pub name: syn::Ident,
    pub mod_name: syn::Ident,
}

impl StructItem {
    pub fn from_item(params: &AttrParams, item: &mut syn::Item) -> Self {
        let data;

        if let syn::Item::Struct(d) = item {
            data = d;
        } else {
            abort! {
                item,
                "Can only be applied to structs"
            }
        }

        let vis = data.vis.clone();
        let name = data.ident.clone();
        let mod_name = format_ident!("clamped_{}", name.to_string().to_case(Case::Snake));

        let ty = &params.integer;

        data.vis = parse_quote!(pub);
        data.fields = syn::Fields::Unnamed(parse_quote! {
            (#ty)
        });

        Self {
            vis,
            name,
            mod_name,
        }
    }
}
