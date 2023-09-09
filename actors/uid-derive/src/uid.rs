use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitInt, Token, Type,
};

use crate::{Expand, Expanded};

/// UID attributes
///
/// #[uid(data = <type>, port = <u32>)]
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    pub data: Option<Type>,
    pub port: Option<LitInt>,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut uid_attrs: Attributes = Default::default();
        while let Ok(key) = input.parse::<Ident>() {
            let _ = input.parse::<Token!(=)>()?;
            if key == "data" {
                uid_attrs.data = input.parse::<Type>().ok();
            }
            if key == "port" {
                uid_attrs.port = input.parse::<LitInt>().ok();
            }
            let Ok(_) = input.parse::<Token!(,)>() else {
                return Ok(uid_attrs);
            };
        }
        Ok(uid_attrs)
    }
}
impl Expand for Attributes {
    fn expand(&self, ident: &Ident) -> Expanded {
        match (&self.data, &self.port) {
            (None, None) => quote! {
                impl ::gmt_dos_clients::interface::UniqueIdentifier for #ident {
                    type DataType = Vec<f64>;
                },
            },
            (None, Some(port)) => quote! {
                impl ::gmt_dos_clients::interface::UniqueIdentifier for #ident {
                    const PORT: u32 = #port;
                    type DataType = Vec<f64>;
                },
            },
            (Some(data), None) => quote! {
                impl ::gmt_dos_clients::interface::UniqueIdentifier for #ident {
                    type DataType = #data;
                },
            },
            (Some(data), Some(port)) => quote! {
                impl ::gmt_dos_clients::interface::UniqueIdentifier for #ident {
                    const PORT: u32 = #port;
                    type DataType = #data;
                },
            },
        }
    }
}
