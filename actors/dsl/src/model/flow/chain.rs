use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

use crate::{client::SharedClient, Expand, Expanded, TryExpand};

pub mod clientoutput;
use clientoutput::{ClientOutputPair, ClientOutputPairMarked, Unbounded};

/// Chain of actors
///
/// A chain consists in a pair of a client and one output
/// A logger may be assigned to a chain in some outputs require to be logged
#[derive(Debug, Clone, Default)]
pub struct Chain {
    pub clientoutput_pairs: Vec<ClientOutputPair>,
    pub logger: Option<SharedClient>,
}

impl Deref for Chain {
    type Target = Vec<ClientOutputPair>;

    fn deref(&self) -> &Self::Target {
        &self.clientoutput_pairs
    }
}
impl DerefMut for Chain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clientoutput_pairs
    }
}

impl Parse for Chain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            clientoutput_pairs:
                Punctuated::<ClientOutputPair, Token![->]>::parse_separated_nonempty(input)?
                    .into_iter()
                    .collect(),
            ..Default::default()
        })
    }
}

impl Chain {
    /// Iteration through the actors chain of each flow replacing duplicated client with shared reference
    pub fn dedup(&mut self, clients: &mut HashSet<SharedClient>) {
        self.iter_mut().for_each(|client_output| {
            let client = &mut client_output.client;
            if !clients.insert(client.clone()) {
                *client = clients.get(client).unwrap().clone();
            }
        });
    }
    /// Iteration through the actors chain of each flow matching output/input rate or setting up a rate transition
    pub fn match_rates(&mut self, flow_rate: usize) {
        let mut iter = self.iter_mut().peekable();
        loop {
            match iter.next() {
                Some(ClientOutputPair {
                    client: output_client,
                    output: Some(output),
                }) => {
                    if let Some(ClientOutputPair {
                        client: input_client,
                        ..
                    }) = iter.peek_mut()
                    {
                        // a client with an output and followed by another client: output_client[output] -> input_client
                        let actor = output_client.actor();
                        let (output_rate, input_rate) = (
                            &mut output_client.borrow_mut().output_rate,
                            &mut input_client.borrow_mut().input_rate,
                        );
                        match (*output_rate > 0, *input_rate > 0) {
                            (true, true) => {}
                            (true, false) => {
                                *input_rate = flow_rate;
                            }
                            (false, true) => {
                                *output_rate = flow_rate;
                            }
                            (false, false) => {
                                *output_rate = flow_rate;
                                *input_rate = flow_rate;
                            }
                        }
                        if *output_rate != *input_rate {
                            output.add_rate_transition(actor, *input_rate, *output_rate);
                        }
                    } else {
                        // a client with an output and not followed by another client: output_client[output]
                        output_client.borrow_mut().output_rate = flow_rate;
                    }
                }
                Some(ClientOutputPair {
                    client: output_client,
                    output: None,
                }) => {
                    // juts a client: output_client
                    if output_client.borrow_mut().input_rate == 0 {
                        output_client.borrow_mut().input_rate = flow_rate
                    };
                }
                None => break,
            }
        }
    }
    /// Check if an output requires logging
    pub fn logging(&self) -> bool {
        self.iter()
            .find(|client_output| {
                if let ClientOutputPair {
                    output: Some(output),
                    ..
                } = client_output
                {
                    output.logging
                } else {
                    false
                }
            })
            .is_some()
    }
}

impl Expand for Chain {
    fn expand(&self) -> Expanded {
        let iter = self
            .iter()
            .skip(1)
            .map(|client_output| client_output.client.actor());
        let outputs: Vec<_> = self
            .iter()
            .zip(iter)
            .filter_map(|(output, input_actor)| {
                if let Some(add_output) = output.try_expand() {
                    Some(quote! {
                        #add_output
                        .into_input(&mut #input_actor)?;
                    })
                } else {
                    None
                }
            })
            .collect();
        if let Some(logger) = self.logger.as_ref() {
            let log_outputs: Vec<_> = self
                .iter()
                .filter_map(|client_output| {
                    client_output
                        .output
                        .as_ref()
                        .and_then(|output| {
                            if output.logging {
                                Some(client_output)
                            } else {
                                None
                            }
                        })
                        .map(|client_output| {
                            let add_output =
                                ClientOutputPairMarked::<Unbounded>::from(client_output).expand();
                            let actor = logger.actor();
                            quote! {
                                #add_output
                                .log(&mut #actor).await?;
                            }
                        })
                })
                .collect();
            quote! {
                #(#outputs)*
                #(#log_outputs)*
            }
        } else {
            quote!(#(#outputs)*)
        }
    }
}
