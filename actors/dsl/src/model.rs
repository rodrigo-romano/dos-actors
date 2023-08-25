use std::collections::HashSet;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, LitInt, LitStr, Token,
};

use crate::{client::SharedClient, Expand, Expanded};

mod flow;
use flow::Flow;

mod keyparam;
use keyparam::{KeyParam, KeyParams};

mod modelstate;
use modelstate::ModelState;

/**
Actors model

A model is a succession of data [Flow]s with [SharedClient]s:

Model
 |- Flow
     |- Chain
         |- ClientOuputPair
             |- SharedClient
             |- Output
 |- SharedClient
*/
#[derive(Debug, Clone, Default)]
pub(super) struct Model {
    pub name: Option<Ident>,
    pub state: ModelState,
    pub flowchart: Option<LitStr>,
    clients: HashSet<SharedClient>,
    flows: Vec<Flow>,
}
impl Model {
    /// Parse model attributes
    ///
    /// #[model(key = param,...)]
    pub fn attributes(mut self, attrs: Option<Vec<Attribute>>) -> syn::Result<Self> {
        let Some(attrs) = attrs else {return Ok(self)};
        for attr in attrs {
            attr.parse_args::<KeyParams>().ok().map(|kps| {
                let _: Vec<_> = kps
                    .into_iter()
                    .map(|kp| {
                        let KeyParam { key, param, .. } = kp;
                        match key.to_string().as_str() {
                            "name" => {
                                self.name = Ident::try_from(&param).ok();
                            }
                            "state" => {
                                self.state = ModelState::try_from(&param)?;
                            }
                            "flowchart" => {
                                self.flowchart = LitStr::try_from(&param).ok();
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    Span::call_site(),
                                    r#"expected model attributes "name" or "state", found {key}"#,
                                ))
                            }
                        }
                        Ok(())
                    })
                    .collect();
            });
        }
        Ok(self)
    }
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut clients = HashSet::new();

        let mut flows: Vec<Flow> = vec![];
        while input.peek(LitInt) {
            flows.push(input.parse()?);
        }

        flows.iter_mut().for_each(|flow| flow.dedup(&mut clients));
        flows.iter_mut().for_each(|flow| flow.match_rates());
        flows
            .iter()
            .for_each(|flow| flow.collect_clients(&mut clients));

        clients.iter().for_each(|client| println!("{client}"));

        Ok(Self {
            clients,
            flows,
            ..Default::default()
        })
    }
}

impl Expand for Model {
    fn expand(&self) -> Expanded {
        let actor_defs: Vec<_> = self.clients.iter().map(|client| client.expand()).collect();
        let flows: Vec<_> = self.flows.iter().map(|flow| flow.expand()).collect();
        let actors: Vec<_> = self.clients.iter().map(|client| client.actor()).collect();
        let (model, name) = match (self.name.clone(), self.flowchart.clone()) {
            (None, None) => {
                let model = Ident::new("model", Span::call_site());
                let name = LitStr::new("model", Span::call_site());
                (model, name)
            }
            (None, Some(name)) => {
                let model = Ident::new("model", Span::call_site());
                (model, name)
            }
            (Some(model), None) => {
                let name = LitStr::new(&model.to_string(), model.span());
                (model, name)
            }
            (Some(model), Some(name)) => (model, name),
        };
        let state = match self.state {
            ModelState::Ready => quote!(.check()?),
            ModelState::Running => quote!(.check()?.run()),
            ModelState::Completed => quote!(.check()?.run().await?),
        };
        quote! {
            // ACTORS DEFINITION
            #(#actor_defs)*
            // ACTORS DEFINITION
            #(#flows)*
            // MODEL
            let #model = ::gmt_dos_actors::prelude::model!(#(#actors),*).name(#name).flowchart()#state;
        }
    }
}
