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
use clientoutput::ClientOutput;

#[derive(Debug, Clone, Default)]
pub struct Chain {
    pub clientoutput_pairs: Vec<ClientOutput>,
    pub logger: Option<SharedClient>,
}

impl Deref for Chain {
    type Target = Vec<ClientOutput>;

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
            clientoutput_pairs: Punctuated::<ClientOutput, Token![->]>::parse_separated_nonempty(
                input,
            )?
            .into_iter()
            .collect(),
            ..Default::default()
        })
    }
}

impl Chain {
    pub fn dedup(&mut self, clients: &mut HashSet<SharedClient>) {
        self.iter_mut().for_each(|client_output| {
            let client = &mut client_output.client;
            if !clients.insert(client.clone()) {
                *client = clients.get(client).unwrap().clone();
            }
        });
    }
    pub fn match_rates(&mut self, flow_rate: usize) {
        let mut iter = self.iter_mut().peekable();
        loop {
            match iter.next() {
                Some(ClientOutput {
                    client: output_client,
                    output: Some(output),
                }) => {
                    if let Some(ClientOutput {
                        client: input_client,
                        ..
                    }) = iter.peek_mut()
                    {
                        let actor = output_client.actor();
                        let (output_rate, input_rate) = (
                            &mut output_client.borrow_mut().output_rate,
                            &mut input_client.borrow_mut().input_rate,
                        );
                        match (*output_rate > 0, *input_rate > 0) {
                            (true, true) => {
                                output.add_rate_transition(actor, *input_rate, *output_rate);
                            }
                            (true, false) => {
                                output.add_rate_transition(actor, flow_rate, *output_rate);
                                *input_rate = flow_rate;
                            }
                            (false, true) => {
                                *output_rate = flow_rate;
                                output.add_rate_transition(actor, *input_rate, flow_rate);
                            }
                            (false, false) => {
                                *output_rate = flow_rate;
                                *input_rate = flow_rate;
                            }
                        }
                    } else {
                        output_client.borrow_mut().output_rate = flow_rate;
                    }
                }
                Some(_) => (),
                None => break,
            }
        }
    }
    /// Check if an output needs logging
    pub fn logging(&self) -> bool {
        self.iter()
            .find(|client_output| {
                if let ClientOutput {
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
                            let add_output = client_output.expand();
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
