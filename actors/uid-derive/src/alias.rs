use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, Type,
};

use crate::{Expand, Expanded};

/// ALIAS attributes
///
/// #[alias(name = <type>, client = <type>, traits = <Write,|Read,|Size>)]
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    name: Option<Type>,
    client: Option<Type>,
    traits: Vec<Ident>,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut alias_attrs: Attributes = Default::default();
        while let Ok(key) = input.parse::<Ident>() {
            let _ = input.parse::<Token!(=)>()?;
            if key == "name" {
                alias_attrs.name = input.parse::<Type>().ok();
            }
            if key == "client" {
                alias_attrs.client = input.parse::<Type>().ok();
            }
            if key == "traits" {
                alias_attrs.traits =
                    Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?
                        .into_iter()
                        .collect();
            }
            let Ok(_) = input.parse::<Token!(,)>() else {
                return Ok(alias_attrs);
            };
        }
        Ok(alias_attrs)
    }
}

impl Expand for Attributes {
    fn expand(&self, ident: &Ident) -> Expanded {
        let Self {
            name,
            client,
            traits,
        } = self;
        let mut write = quote!();
        let mut read = quote!();
        let mut size = quote!();
        for a_trait in traits {
            if a_trait.to_string().as_str().to_lowercase() == "write" {
                write = quote! {
                    impl ::interface::Write<#ident> for #client {
                        fn write(&mut self) -> Option<::interface::Data<#ident>> {
                            let mut data: ::interface::Data<#name> = self.write()?;
                            Some(data.transmute())
                        }
                    }
                };
            }
            if a_trait.to_string().as_str().to_lowercase() == "read" {
                read = quote! {
                    impl ::interface::Read<#ident> for #client {
                        fn read(&mut self,data: ::interface::Data<#ident>) {
                            <Self as ::interface::Read<#name>>::read(self,data.transmute());
                        }
                    }
                };
            }
            if a_trait.to_string().as_str().to_lowercase() == "size" {
                size = quote! {
                    impl ::interface::Size<#ident> for #client {
                        fn len(&self) -> usize {
                            <Self as ::interface::Size<#name>>::len(self)
                        }
                    }
                };
            }
        }
        quote! {
            #write
            #read
            #size
        }
    }
}
