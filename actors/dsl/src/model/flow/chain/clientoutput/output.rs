use std::collections::HashSet;

use proc_macro2::{Delimiter, Span};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token::Bracket,
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
    #[allow(dead_code)]
    pub fn name(&self) -> Ident {
        self.name.clone()
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
    pub fn add_option(&mut self, option: &str) {
        self.options
            .get_or_insert(vec![])
            .push(Ident::new(option, Span::call_site()));
    }
    pub fn add_logging(&mut self) {
        self.logging = true;
    }
}

impl Parse for Output {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // looking for an output name within brackets i.e. client[output_name]
        input
            .peek(Bracket)
            .then(|| {
                let content;
                let _ = bracketed!(content in input);
                let mut output = Output::new(content.parse::<Ident>()?);
                // checking out for output options
                while let Ok(id) = input.parse::<Token![!]>().map_or_else(
                    |_err| input.parse::<Token![$]>().map(|_| OutputOptions::Logger),
                    |_not| Ok(OutputOptions::Bootstrap),
                ) {
                    match id {
                        OutputOptions::Bootstrap => {
                            output.add_option("bootstrap");
                        }
                        OutputOptions::Logger => {
                            output.add_logging();
                        }
                        _ => todo!(),
                    }
                }
                Ok(output)
            })
            .ok_or(syn::Error::new(input.span(), "no output given "))
            .and_then(|maybe_output| maybe_output)
    }
}
