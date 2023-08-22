use std::collections::HashSet;

use proc_macro2::{Delimiter, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token,
};

use crate::{
    client::{ClientKind, SharedClient},
    Expand, Expanded, TryExpand,
};

#[derive(Debug, Clone)]
enum OutputOptions {
    Bootstrap,
    Logger,
}

#[derive(Debug, Clone)]
pub struct Output {
    // output type
    pub name: Ident,
    // ouput options: bootstrap, unbounded
    pub options: Option<Vec<Ident>>,
    // need a rate transition
    pub rate_transition: Option<SharedClient>,
    // extra clients
    pub extras: Option<Vec<ClientKind>>,
    pub logging: bool,
}
impl Output {
    pub fn new(name: Ident) -> Self {
        Self {
            name,
            options: None,
            rate_transition: None,
            extras: None,
            logging: false,
        }
    }
    /// Clone and collect all the clients
    pub fn collect(&self, clients: &mut HashSet<SharedClient>) {
        self.rate_transition
            .as_ref()
            .map(|client| clients.insert(client.clone()));
    }
    /// Add a rate transition sampler client
    pub fn add_rate_transition(&mut self, actor: Ident, output_rate: usize, input_rate: usize) {
        self.rate_transition = Some(SharedClient::sampler(
            actor,
            self.name.clone(),
            output_rate,
            input_rate,
        ));
    }
}

impl Parse for Output {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input
            .step(|stream| {
                stream
                    .group(Delimiter::Bracket)
                    .and_then(|(content, _span, rest)| {
                        content.ident().map(|(ident, ..)| (ident, rest))
                    })
                    .ok_or(stream.error("actor w/o output"))
            })
            .map(|name| Output::new(name))
            .map(|mut output| {
                while let Ok(id) = input.parse::<Token![!]>().map_or_else(
                    |_err| input.parse::<Token![$]>().map(|_| OutputOptions::Logger),
                    |_not| Ok(OutputOptions::Bootstrap),
                ) {
                    match id {
                        OutputOptions::Bootstrap => {
                            output
                                .options
                                .get_or_insert(vec![])
                                .push(Ident::new("bootstrap", Span::call_site()));
                        }
                        OutputOptions::Logger => output.logging = true,
                    }
                }
                output
            })
    }
}

#[derive(Debug, Clone)]
pub struct ClientOutput {
    pub client: SharedClient,
    pub output: Option<Output>,
}

impl Parse for ClientOutput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let reference = input.parse::<Token![&]>().is_ok();
        Ok(Self {
            client: SharedClient::new(input.parse()?, reference),
            output: input.parse::<Output>().ok(),
        })
    }
}

impl TryExpand for ClientOutput {
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

impl Expand for ClientOutput {
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
