use std::{marker::PhantomData, ops::Deref};

use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Ident, LitStr, Token,
};

use crate::{client::SharedClient, Expand, Expanded, TryExpand};

mod output;
use output::Output;

/// A pair of a client and one ouput
#[derive(Debug, Clone)]
pub struct ClientOutputPair {
    pub client: SharedClient,
    pub output: Option<Output>,
}

impl Parse for ClientOutputPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let reference = input.parse::<Token![&]>().is_ok();
        let name: Ident = input.parse()?;
        let label = input
            .peek(Paren)
            .then(|| {
                let content;
                let _ = parenthesized!(content in input);
                let label: LitStr = content.parse()?;
                Ok(label)
            })
            .transpose()
            .ok()
            .flatten();
        Ok(Self {
            client: SharedClient::new(name, reference, label),
            output: input.parse::<Output>().ok(),
        })
    }
}

impl TryExpand for ClientOutputPair {
    fn try_expand(&self) -> Option<Expanded> {
        if let Some(output) = self.output.as_ref() {
            let actor = self.client.actor();
            let Output {
                options,
                rate_transition,
                ..
            } = output;
            let name = output.expand_name();
            Some(match (options, rate_transition) {
                (None, None) => quote! {
                    #actor
                    .add_output()
                    .build::<#name>()
                },
                (None, Some(client)) => {
                    let sampler = client.actor();
                    quote! {
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

impl Expand for ClientOutputPair {
    fn expand(&self) -> Expanded {
        let output = self.output.as_ref().unwrap();
        let actor = self.client.actor();
        let Output { options, .. } = output;
        let name = output.expand_name();
        match options {
            None => quote! {
                #actor
                .add_output()
                .build::<#name>()
            },

            Some(options) => quote! {
                #actor
                .add_output()
                #(.#options())*
                .build::<#name>()
            },
        }
    }
}

pub struct ClientOutputPairMarked<'a, M>(&'a ClientOutputPair, PhantomData<&'a M>);
impl<'a, M> Deref for ClientOutputPairMarked<'a, M> {
    type Target = ClientOutputPair;

    fn deref(&self) -> &'a Self::Target {
        &self.0
    }
}
impl<'a, M> From<&'a ClientOutputPair> for ClientOutputPairMarked<'a, M> {
    fn from(value: &'a ClientOutputPair) -> Self {
        ClientOutputPairMarked(value, PhantomData)
    }
}
pub enum Unbounded {}
impl<'a> Expand for ClientOutputPairMarked<'a, Unbounded> {
    fn expand(&self) -> Expanded {
        let output = self.output.as_ref().unwrap();
        let actor = self.client.actor();
        let Output { options, .. } = output;
        let name = output.expand_name();
        match options {
            None => quote! {
                #actor
                .add_output()
                .unbounded()
                .build::<#name>()
            },

            Some(options) => quote! {
                #actor
                .add_output()
                .unbounded()
                #(.#options())*
                .build::<#name>()
            },
        }
    }
}
