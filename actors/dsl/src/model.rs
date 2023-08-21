use std::collections::HashSet;

use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

use crate::{client::SharedClient, Expand, Expanded};

mod flow;
use flow::Flow;

#[derive(Debug, Clone)]
struct ParentheSizedFlow(Flow);

impl Parse for ParentheSizedFlow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let flow = content.parse()?;
        Ok(Self(flow))
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct Model {
    // pub name: Option<Ident>,
    // pub state: Option<Ident>,
    clients: HashSet<SharedClient>,
    flows: Vec<Flow>,
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut clients = HashSet::new();
        let content;
        let _ = braced!(content in input);
        let mut flows: Vec<_> = content
            .parse_terminated(ParentheSizedFlow::parse, Token!(,))?
            .into_iter()
            .map(|parenthesized_flow| parenthesized_flow.0)
            .collect();

        flows.iter_mut().for_each(|flow| {
            flow.dedup(&mut clients);
        });

        flows.iter_mut().for_each(|flow| {
            flow.match_rates(&mut clients);
        });

        Ok(Self { clients, flows })
    }
}

impl Expand for Model {
    fn expand(&self) -> Expanded {
        let actor_defs: Vec<_> = self.clients.iter().map(|client| client.expand()).collect();
        let flows: Vec<_> = self.flows.iter().map(|flow| flow.expand()).collect();
        let actors: Vec<_> = self.clients.iter().map(|client| client.actor()).collect();
        quote! {
            // ACTORS DEFINITION
            #(#actor_defs)*
            // ACTORS DEFINITION
            #(#flows)*
            // MODEL
            let model = ::gmt_dos_actors::prelude::model!(#(#actors),*);
        }
    }
}
