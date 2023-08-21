use proc_macro2::Span;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token::Bracket,
    GenericParam, Generics, Ident, Token, TypeParam,
};

use crate::{client::SharedClient, Expanded, TryExpand};

#[derive(Debug, Clone)]
pub struct Output {
    // output type
    pub name: Ident,
    // ouput options: bootstrap, unbounded
    pub options: Option<Vec<Ident>>,
    // need a rate transition
    pub rate_transition: Option<SharedClient>,
}

#[derive(Debug, Clone)]
pub struct ClientOutput(pub SharedClient, pub Option<Output>);

impl Parse for ClientOutput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let reference = input.parse::<Token![&]>().is_ok();
        let name: Ident = input.parse()?;
        let output = input
            .peek(Bracket) // looking for an opening bracket: [
            .then(|| {
                let content;
                let _ = bracketed!(content in input); // getting the content within [...,...,...]
                Ok(content
                    .parse_terminated(Ident::parse, Token!(,))?
                    .into_iter()
                    .collect::<Vec<_>>()) // parsing the coma separated content
            })
            .transpose()
            .ok()
            .zip(input.parse::<Generics>().ok().and_then(|generics| {
                // parsing the output type identifer
                generics.params.into_iter().next().map(|g| {
                    let GenericParam::Type(TypeParam { ident, .. }) = g else { todo!() };
                    ident
                })
            }))
            .map(|(options, name)| Output {
                name,
                options,
                rate_transition: None,
            });
        let actor = if reference {
            Ident::new(&format!("{name}_actor"), Span::call_site())
        } else {
            name.clone()
        };
        Ok(Self(SharedClient::new(name, actor, reference), output))
    }
}

impl TryExpand for ClientOutput {
    fn try_expand(&self) -> Option<Expanded> {
        if let Self(client, Some(output), ..) = self {
            let actor = client.actor();
            let Output {
                name,
                options,
                rate_transition,
            } = output;
            Some(match (options, rate_transition) {
                (None, None) => quote! {
                    #actor
                    .add_output()
                    .build::<#name>()
                },
                (None, Some(client)) => {
                    let sampler = client.actor();
                    // let output_rate = client.lit_output_rate();
                    // let input_rate = client.lit_input_rate();
                    quote! {
                        // let mut #sampler: ::gmt_dos_actors::prelude::Actor::<_,#output_rate,#input_rate> = ::gmt_dos_clients::Sampler::default().into();
                        #actor
                        .add_output()
                        .build::<#name>()
                        .into_input(&mut #sampler)?;
                        #sampler
                        .add_output()
                        .build::<#name>()
                    }
                }
                (Some(options), None) => quote! {
                    #actor
                    .add_output()
                    #(.#options())*
                    .build::<#name>()
                },
                (Some(options), Some(client)) => {
                    let sampler = client.actor();
                    quote! {
                            #actor
                            .add_output()
                            #(.#options())*
                            .build::<#name>()
                            .into_input(&mut #sampler)?;
                            #sampler
                            .add_output()
                            .build::<#name>()
                    }
                }
            })
        } else {
            None
        }
    }
}
