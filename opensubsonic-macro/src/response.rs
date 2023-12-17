pub fn expand(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemStruct);
    insert_serialize_if_attr(&mut input);

    From::from(quote::quote! {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #input
    })
}

fn insert_serialize_if_attr(input: &mut syn::ItemStruct) {
    for field in input.fields.iter_mut() {
        // this should only work for 'field: Option<T>'
        let is_option = match &field.ty {
            syn::Type::Path(p) => p
                .path
                .segments
                .first()
                .map(|s| s.ident == "Option")
                .unwrap_or(false),
            _ => false,
        };

        if is_option {
            field
                .attrs
                .push(syn::parse_quote!(#[serde(skip_serializing_if = "Option::is_none")]));
        }
    }
}
