use std::fmt::Display;

use proc_macro2::Span;
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token::{Brace, Paren},
    Ident, LitStr, Token,
};

use crate::{client::SharedClient, Expanded, TryExpand};

mod output;
pub use output::{MaybeOutput, Output};

/// A pair of a client and one ouput
#[derive(Debug, Clone)]
pub struct ClientOutputPair {
    pub client: SharedClient,
    pub output: Option<Output>,
}

impl Display for ClientOutputPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(output) = &self.output {
            if let Some(rate_transition) = &output.rate_transition {
                write!(
                    f,
                    "{}{} -> {}",
                    self.client.actor(),
                    output,
                    rate_transition.actor()
                )
            } else {
                write!(f, "{}{}", self.client.actor(), output)
            }
        } else {
            write!(f, "{}", self.client.actor())
        }
    }
}

impl From<SharedClient> for ClientOutputPair {
    fn from(client: SharedClient) -> Self {
        Self {
            client,
            output: None,
        }
    }
}

impl Parse for ClientOutputPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let label = |input: ParseStream| {
            input
                .peek(Paren)
                .then(|| {
                    let content;
                    let _ = parenthesized!(content in input);
                    let label: LitStr = content.parse()?;
                    Ok(label)
                })
                .transpose()
                .ok()
                .flatten()
        };
        let reference = input.parse::<Token![&]>().is_ok();
        let client = if input.peek(Brace) {
            let content;
            let _ = braced!(content in input);
            let name: Ident = content.parse()?;
            SharedClient::subsystem(name, reference, label(input))
        } else {
            let name: Ident = input.parse()?;
            SharedClient::new(name, reference, label(input))
        };
        Ok(Self {
            client,
            output: input.parse::<MaybeOutput>()?.into_inner(),
        })
    }
}

impl TryExpand for ClientOutputPair {
    fn try_expand(&self) -> syn::Result<Expanded> {
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
        .ok_or(syn::Error::new(Span::call_site(), "no output to quote"))
    }
}
