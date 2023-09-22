use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    DeriveInput, Ident, Token, Type,
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
    pub skip_uid: bool,
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
    fn expand(&self, input: &DeriveInput) -> Expanded {
        let Self {
            name,
            client,
            traits,
            skip_uid,
        } = self;
        let DeriveInput {
            ident, generics, ..
        } = input;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let uid = if *skip_uid {
            quote!()
        } else {
            quote! {
                impl #impl_generics ::interface::UniqueIdentifier for #ident #ty_generics #where_clause {
                    const PORT: u32 = <#name as ::interface::UniqueIdentifier>::PORT;
                    type DataType = <#name as ::interface::UniqueIdentifier>::DataType;
                }
            }
        };
        let mut write = quote!();
        let mut read = quote!();
        let mut size = quote!();
        for a_trait in traits {
            if a_trait.to_string().as_str().to_lowercase() == "write" {
                write = quote! {
                    impl #impl_generics ::interface::Write<#ident #ty_generics> for #client {
                        #[inline]
                        fn write(&mut self) -> Option<::interface::Data<#ident #ty_generics>> {
                            <Self as ::interface::Write<#name>>::write(self).map(|data| data.transmute())
                        }
                    }
                };
            }
            if a_trait.to_string().as_str().to_lowercase() == "read" {
                read = quote! {
                    impl #impl_generics ::interface::Read<#ident #ty_generics> for #client {
                        #[inline]
                        fn read(&mut self,data: ::interface::Data<#ident #ty_generics>) {
                            <Self as ::interface::Read<#name>>::read(self,data.transmute());
                        }
                    }
                };
            }
            if a_trait.to_string().as_str().to_lowercase() == "size" {
                size = quote! {
                    impl #impl_generics ::interface::Size<#ident #ty_generics> for #client {
                        #[inline]
                        fn len(&self) -> usize {
                            <Self as ::interface::Size<#name>>::len(self)
                        }
                    }
                };
            }
        }
        quote! {
            #uid
            #write
            #read
            #size
        }
    }
}
