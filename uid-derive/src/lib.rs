use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Lit, Meta, NestedMeta};

#[proc_macro_derive(UID, attributes(uid))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident: proc_macro2::Ident = input.ident;
    let attr: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("uid"))
        .collect();
    {
        if attr.is_empty() {
            Ok(quote! {
            impl UniqueIdentifier for #ident {
                type Data = Vec<f64>;
            }
            })
        } else {
            get_attr(attr[0]).map(|data| {
                quote! {
                impl UniqueIdentifier for #ident {
                    type Data = #data;
                }
                }
            })
        }
    }
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

fn get_attr(attr: &Attribute) -> syn::Result<syn::TypePath> {
    let meta = attr.parse_meta()?;
    match meta {
        Meta::List(list) => {
            if list.nested.len() == 1 {
                let nested = &list.nested[0];
                match nested {
                    NestedMeta::Meta(Meta::NameValue(nv)) => {
                        if nv.path.is_ident("data") {
                            if let Lit::Str(ref val) = nv.lit {
                                val.parse()
                            } else {
                                Err(syn::Error::new_spanned(&nv.lit, "expected String litteral"))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                &nv.path,
                                "expected `data` as uid attribute",
                            ))
                        }
                    }
                    _ => Err(syn::Error::new_spanned(
                        nested,
                        "expected `name = \"<value>\"` argument",
                    )),
                }
            } else {
                Err(syn::Error::new_spanned(
                    list,
                    "expected only a single attribute",
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            meta,
            "expected a list of attributes",
        )),
    }
}
