use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Token,
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
        Ok(Self {
            client: SharedClient::new(input.parse()?, reference),
            output: input.parse::<Output>().ok(),
        })
    }
}

impl TryExpand for ClientOutputPair {
    fn try_expand(&self) -> Option<Expanded> {
        if let Some(output) = self.output.as_ref() {
            let actor = self.client.actor();
            let Output {
                name,
                options,
                rate_transition,
                ..
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

impl Expand for ClientOutputPair {
    fn expand(&self) -> Expanded {
        let output = self.output.as_ref().unwrap();
        let actor = self.client.actor();
        let Output { name, options, .. } = output;
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
