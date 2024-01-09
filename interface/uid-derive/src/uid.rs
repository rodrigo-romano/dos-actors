use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Generics, Ident, LitInt, Token,
};

use crate::{Expand, Expanded};

/// UID attributes
///
/// #[uid(data = <type>, port = <u32>)]
#[derive(Debug, Clone)]
pub struct Attributes {
    pub ident: Ident,
    pub port: LitInt,
    generics: Generics,
}

impl Default for Attributes {
    fn default() -> Self {
        let ident: Ident = syn::parse2(quote!(Vec)).unwrap();
        let generics: Generics = syn::parse2(quote!(<f64>)).unwrap();
        Self {
            ident,
            port: LitInt::new("50_000", Span::call_site()),
            generics,
        }
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut uid_attrs: Attributes = Default::default();
        while let Ok(key) = input.parse::<Ident>() {
            let _ = input.parse::<Token!(=)>()?;
            if key == "data" {
                uid_attrs.ident = input.parse::<Ident>()?;
                uid_attrs.generics = input.parse::<Generics>()?;
            }
            if key == "port" {
                let _ = input.parse::<LitInt>().map(|port| {
                    uid_attrs.port = port;
                });
            }
            let Ok(_) = input.parse::<Token!(,)>() else {
                return Ok(uid_attrs);
            };
        }
        Ok(uid_attrs)
    }
}
impl Expand for Attributes {
    fn expand(&self, input: &DeriveInput) -> Expanded {
        let DeriveInput {
            ident, generics, ..
        } = input;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let Self {
            ident: name,
            port,
            generics: name_generics,
        } = self;
        let (_name_impl_generics, name_ty_generics, _name_where_clause) =
            name_generics.split_for_impl();
        quote! {
            impl #impl_generics ::interface::UniqueIdentifier for #ident #ty_generics #where_clause {
                const PORT: u32 = #port;
                type DataType = #name #name_ty_generics;
            }
        }
    }
}
