use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseBuffer, ParseStream},
    token::Bracket,
    Generics, Ident, Token,
};

use crate::client::{ClientKind, SharedClient};

/// Actor ouput
#[derive(Debug, Clone)]
pub struct Output {
    // output type
    pub name: Ident,
    pub generics: Option<Generics>,
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
    pub fn new(name: Ident, generics: Option<Generics>) -> Self {
        Self {
            name,
            generics,
            options: None,
            rate_transition: None,
            extras: None,
            logging: false,
        }
    }
    pub fn expand_name(&self) -> TokenStream {
        let name = &self.name;
        if let Some(generics) = self.generics.as_ref() {
            quote!(#name #generics)
        } else {
            quote!(#name)
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
    pub fn add_option(&mut self, option: &str) {
        self.options
            .get_or_insert(vec![])
            .push(Ident::new(option, Span::call_site()));
    }
    pub fn add_logging(&mut self) {
        self.logging = true;
    }
}
impl<'a> TryFrom<ParseBuffer<'a>> for Output {
    type Error = syn::parse::Error;

    fn try_from(content: ParseBuffer<'a>) -> Result<Self, Self::Error> {
        Ok(Output::new(
            content.parse::<Ident>()?,
            content.parse::<Generics>().ok(),
        ))
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
                let mut output = Output::try_from(content)?;
                // checking out for output options either !, .. or $ ,
                // or any combination of the 3 after the output i.e.
                // client[output_name]!$
                loop {
                    match (
                        input.peek(Token![!]),
                        input.peek(Token![$]),
                        input.peek(Token![..]),
                    ) {
                        (true, false, false) => {
                            input
                                .parse::<Token![!]>()
                                .map(|_| output.add_option("bootstrap"))?;
                        }
                        (false, false, true) => {
                            input
                                .parse::<Token![..]>()
                                .map(|_| output.add_option("unbounded"))?;
                        }
                        (false, true, false) => {
                            input.parse::<Token![$]>().map(|_| output.add_logging())?;
                        }
                        (false, false, false) => break,
                        _ => unimplemented!(),
                    }
                }
                Ok(output)
            })
            .ok_or(syn::Error::new(input.span(), "no output given "))
            .and_then(|maybe_output| maybe_output)
    }
}
