use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use crate::{client::SharedClient, Expand, Expanded, TryExpand};

mod clientoutput;
use clientoutput::ClientOutput;

#[derive(Debug, Clone)]
pub struct Chain(Vec<ClientOutput>);

impl Deref for Chain {
    type Target = Vec<ClientOutput>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Chain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Parse for Chain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input
                .parse_terminated(ClientOutput::parse, Token![->])?
                .into_iter()
                .collect(),
        ))
    }
}

impl Chain {
    pub fn dedup(&mut self, clients: &mut HashSet<SharedClient>) {
        self.iter_mut().for_each(|ClientOutput(client, ..)| {
            if !clients.insert(client.clone()) {
                *client = clients.get(client).unwrap().clone();
            }
        });
    }
    pub fn match_rates(&mut self, flow_rate: usize) -> Option<Vec<SharedClient>> {
        let mut iter = self.iter_mut().peekable();
        let mut samplers = Option::<Vec<SharedClient>>::None;
        loop {
            match iter.next() {
                Some(ClientOutput(output_client, Some(output))) => {
                    if let Some(ClientOutput(input_client, ..)) = iter.peek_mut() {
                        let actor = output_client.actor();
                        let (output_rate, input_rate) = (
                            &mut output_client.borrow_mut().output_rate,
                            &mut input_client.borrow_mut().input_rate,
                        );
                        match (*output_rate > 0, *input_rate > 0) {
                            (true, true) => {
                                let sampler = SharedClient::sampler(
                                    actor,
                                    output.name.clone(),
                                    *input_rate,
                                    *output_rate,
                                );
                                output.rate_transition = Some(sampler.clone());
                                if let Some(samplers) = samplers.as_mut() {
                                    samplers.push(sampler.clone());
                                } else {
                                    samplers = Some(vec![sampler.clone()]);
                                }
                            }
                            (true, false) => {
                                let sampler = SharedClient::sampler(
                                    actor,
                                    output.name.clone(),
                                    flow_rate,
                                    *output_rate,
                                );
                                output.rate_transition = Some(sampler.clone());
                                if let Some(samplers) = samplers.as_mut() {
                                    samplers.push(sampler.clone());
                                } else {
                                    samplers = Some(vec![sampler.clone()]);
                                }
                                *input_rate = flow_rate;
                            }
                            (false, true) => {
                                *output_rate = flow_rate;
                                let sampler = SharedClient::sampler(
                                    actor,
                                    output.name.clone(),
                                    *input_rate,
                                    flow_rate,
                                );
                                output.rate_transition = Some(sampler.clone());
                                if let Some(samplers) = samplers.as_mut() {
                                    samplers.push(sampler.clone());
                                } else {
                                    samplers = Some(vec![sampler.clone()]);
                                }
                            }
                            (false, false) => {
                                *output_rate = flow_rate;
                                *input_rate = flow_rate;
                            }
                        }
                    }
                }
                Some(ClientOutput(_client, None)) => (),
                None => break,
            }
        }
        samplers
    }
}

impl Expand for Chain {
    fn expand(&self) -> Expanded {
        let iter = self
            .iter()
            .skip(1)
            .map(|ClientOutput(client, ..)| client.actor());
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
        quote!(#(#outputs)*)
    }
}
