use std::collections::HashSet;

use syn::{
    parse::{Parse, ParseStream},
    LitInt, Token,
};

use crate::{client::SharedClient, Expand, Expanded};

mod chain;
use chain::Chain;

/// Data flow
///
/// A flow is characterized by a sampling rate and
/// a chain of actors i.e
/// actor1[output1_of_actor1] -> actor2[output1_of_actor2] -> actor3
#[derive(Debug, Clone, Default)]
pub struct Flow {
    pub rate: usize,
    pub chain: Chain,
}

impl Flow {
    /// Iteration through the actors chain of each flow replacing duplicated client with shared reference
    pub fn dedup(&mut self, clients: &mut HashSet<SharedClient>) {
        self.chain.dedup(clients);
    }
    /// Iteration through the actors chain of each flow matching output/input rate or setting up a rate transition
    pub fn match_rates(&mut self) {
        self.chain.match_rates(self.rate);
    }
    /// Collect output special clients
    ///
    /// Special clients are [Sampler], [Logger]
    pub fn collect_clients(&self, clients: &mut HashSet<SharedClient>) {
        self.chain.iter().for_each(|client_output| {
            client_output
                .output
                .as_ref()
                .map(|output| output.collect(clients));
            self.chain
                .logger
                .as_ref()
                .map(|logger| clients.insert(logger.clone()));
        });
    }
}

impl Parse for Flow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let rate = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        let _: Token!(:) = input.parse()?;
        let mut chain: Chain = input.parse()?;
        chain.logging().then(|| {
            let logger = SharedClient::logger(rate);
            chain.logger = Some(logger.clone());
        });
        Ok(Self { rate, chain })
    }
}

impl Expand for Flow {
    fn expand(&self) -> Expanded {
        self.chain.expand()
    }
}
