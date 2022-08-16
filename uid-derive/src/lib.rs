use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Lit, Meta, NestedMeta};

#[proc_macro_derive(UID, attributes(uid, alias))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident: proc_macro2::Ident = input.ident;
    let attrs: Vec<_> = input
        .attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("uid") || attr.path.is_ident("alias"))
        .collect();
    let token = match attrs.len() {
        n if n == 0 => Ok(quote! {
        impl uid::UniqueIdentifier for #ident {
            type Data = Vec<f64>;
        }
        })
        .map(|token| token.into()),
        n if n == 1 => {
            let attr = &attrs[0];
            match attr.path.get_ident() {
                Some(id) if id == "uid" => get_data_type(attr)
                    .map(|data| {
                        quote! {
                        impl uid::UniqueIdentifier for #ident {
                            type Data = #data;
                        }
                        }
                    })
                    .map(|token| token.into()),
                Some(id) if id == "alias" => {
                    get_name_client_traits(attr).and_then(|alias| alias.token(ident))
                }
                _ => Err(syn::Error::new_spanned(
                    attr,
                    "expected only a single attribute",
                )),
            }
        }
        _ => Err(syn::Error::new(
            Span::mixed_site(),
            "expected only a single input",
        )),
    };
    match token {
        Ok(token) => token,
        Err(e) => e.into_compile_error().into(),
    }
}
fn get_data_type(attr: &Attribute) -> syn::Result<syn::TypePath> {
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

struct Alias {
    name: syn::Result<syn::TypePath>,
    client: Client,
}
struct Client {
    name: syn::Result<syn::TypePath>,
    traits: syn::Result<String>,
}
impl Alias {
    fn token(self, ident: Ident) -> syn::Result<TokenStream> {
        self.name
            .and_then(|name| {
                if let (Ok(client), Ok(traits)) = (self.client.name, self.client.traits) {
                    traits
                    .split(',')
                    .map(|t| match t {
                        "Write" => Ok(quote! {
                            impl Write<<#name as uid::UniqueIdentifier>::Data,#ident> for #client {
                                fn write(&mut self) -> Option<Arc<Data<#ident>>> {
                                    let mut data: Arc<Data<#name>> = self.write()?;
                                    let inner = Arc::get_mut(&mut data)?;
                                    Some(Arc::new(inner.into()))
                                }
                            }
                        }),
                        "Read" => unimplemented!(),
                        "Size" => Ok(quote! {
                            impl Size<#ident> for #client {
                                fn len(&self) -> usize {
                                    <Self as Size<#name>>::len(self)
                                }
                            }
                        }),
                        _ => Err(syn::Error::new(Span::mixed_site(), "missing alias client")),
                    })
                    .collect::<syn::Result<Vec<_>>>()
                } else {
                    Err(syn::Error::new(Span::mixed_site(), "missing alias client"))
                }
                .map(|client_token| {
                    quote! {
                    impl uid::UniqueIdentifier for #ident {
                        type Data = <#name as uid::UniqueIdentifier>::Data;
                    }
                    #(#client_token)*
                    }
                })
            })
            .map(|token| token.into())
    }
}

fn get_name_client_traits(attr: &Attribute) -> syn::Result<Alias> {
    let client = Client {
        name: Err(syn::Error::new(Span::mixed_site(), "missing alias name")),
        traits: Err(syn::Error::new(Span::mixed_site(), "missing alias client")),
    };
    let mut alias = Alias {
        name: Err(syn::Error::new(Span::mixed_site(), "missing alias name")),
        client,
    };

    let meta = attr.parse_meta()?;
    match meta {
        Meta::List(list) => {
            for nested in list.nested.iter() {
                match nested {
                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                        alias.name = if let Lit::Str(ref val) = nv.lit {
                            val.parse()
                        } else {
                            Err(syn::Error::new_spanned(&nv.lit, "expected String litteral"))
                        };
                        Ok(())
                    }
                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("client") => {
                        alias.client.name = if let Lit::Str(ref val) = nv.lit {
                            val.parse()
                        } else {
                            Err(syn::Error::new_spanned(&nv.lit, "expected String litteral"))
                        };
                        Ok(())
                    }
                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("traits") => {
                        alias.client.traits = if let Lit::Str(ref val) = nv.lit {
                            Ok(val.value())
                        } else {
                            Err(syn::Error::new_spanned(&nv.lit, "expected String litteral"))
                        };
                        Ok(())
                    }
                    _ => Err(syn::Error::new_spanned(
                        nested,
                        "expected `name = \"<value>\"` argument",
                    )),
                }?;
            }
            Ok(alias)
        }
        _ => Err(syn::Error::new_spanned(
            meta,
            "expected a list of attributes",
        )),
    }
}
