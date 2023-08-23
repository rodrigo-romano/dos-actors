use std::collections::HashSet;

use proc_macro2::{Delimiter, Span};
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token,
};

use crate::client::{ClientKind, SharedClient};

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[non_exhaustive]
enum OutputOptions {
    Bootstrap,
    Logger,
    Transmitter,
    Receiver,
    Scope,
}

/// Actor ouput
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
    /// Creates a new output
    pub fn new(name: Ident) -> Self {
        Self {
            name,
            options: None,
            rate_transition: None,
            extras: None,
            logging: false,
        }
    }
    /// Clone and collect any sampler clients
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
        /*         input
        .cursor()
        .group(Delimiter::Bracket)
        .and_then(|(content, _span, rest)| content.ident().map(|(ident, ..)| (ident, rest)))
        .ok_or_else(||input.error("actor w/o output")); */
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
                        _ => todo!(),
                    }
                }
                output
            })
    }
}
