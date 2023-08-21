use std::collections::HashSet;

use syn::{
    parse::{Parse, ParseStream},
    LitInt, Token,
};

use crate::{client::SharedClient, Expand, Expanded};

mod chain;
use chain::Chain;

#[derive(Debug, Clone)]
pub struct Flow {
    pub rate: usize,
    pub chain: Chain,
}

impl Flow {
    /// Iteration through the actors chain of each flow replacing duplicated client with shared reference
    pub fn dedup(&mut self, clients: &mut HashSet<SharedClient>) -> &mut Self {
        self.chain.dedup(clients);
        self
    }
    /// Iteration through the actors chain of each flow matching output/input rate or setting up a rate transition
    pub fn match_rates(&mut self, clients: &mut HashSet<SharedClient>) -> &mut Self {
        let samplers = self.chain.match_rates(self.rate);
        samplers.map(|samplers| {
            samplers.into_iter().for_each(|sampler| {
                clients.insert(sampler);
            })
        });
        self
    }
}

impl Parse for Flow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let rate: LitInt = input.parse()?;
        let _: Token!(:) = input.parse()?;
        Ok(Self {
            rate: rate.base10_parse::<usize>()?,
            chain: input.parse()?,
        })
    }
}

impl Expand for Flow {
    fn expand(&self) -> Expanded {
        self.chain.expand()
    }
}
