#![allow(unused)]
use syn::Result;

use crate::attr;

struct ContainerAttributes {}

impl ContainerAttributes {
    fn extract(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let metas = attr::extract_meta_list(attrs)?;

        // for meta in metas {
        //     match &meta {
        //         _ => return Err(syn::Error::new_spanned(meta, "Invalid subsonic attribute")),
        //     }
        // }

        Ok(Self {})
    }
}

pub fn expand(mut input: syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let _container_attrs = ContainerAttributes::extract(&mut input.attrs)?;
    let container_ident = &input.ident;

    let output = quote::quote! {
        impl crate::request::SubsonicRequest for #container_ident {
        }
    };

    Ok(output)
}
