use std::fmt::Display;

use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Ident, LitStr, Token,
};

use crate::{client::SharedClient, Expanded, TryExpand};

mod output;
pub use output::Output;

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
