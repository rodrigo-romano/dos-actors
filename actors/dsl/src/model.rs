use std::{collections::HashSet, fmt::Display};

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, LitInt, LitStr,
};

use crate::{client::SharedClient, Expand, Expanded, TryExpand};

mod flow;
use flow::Flow;

mod keyparam;
use keyparam::{KeyParam, KeyParams};

mod modelstate;
use modelstate::ModelState;

mod scope;
pub use scope::{Scope, ScopeSignal};

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
    pub clients: HashSet<SharedClient>,
    pub flows: Vec<Flow>,
    pub scope: Scope,
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Model {} [{}] :",
            self.name().to_string().to_uppercase(),
            self.state
        )?;
        for flow in self.flows.iter() {
            writeln!(f, "{flow}")?;
        }
        Ok(())
    }
}

impl Model {
    /// Returns the model name
    pub fn name(&self) -> Ident {
        self.name
            .clone()
            .unwrap_or_else(|| Ident::new("model", Span::call_site()))
    }
    /// Parse model attributes
    ///
    /// #[model(key = param,...)]
    pub fn attributes(mut self, attrs: Option<Vec<Attribute>>) -> syn::Result<Self> {
        let Some(attrs) = attrs else { return Ok(self) };
        for attr in attrs {
            match attr
                .path()
                .get_ident()
                .map(|id| id.to_string())
                .as_ref()
                .map(|s| s.as_str())
            {
                Some("model") => {
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
                Some("scope") => (),
                _ => unimplemented!(),
            }
        }
        Ok(self)
    }
    /// Build the model
    pub fn build(mut self) -> Self {
        let mut flow_implicits: Vec<_> = self
            .flows
            .iter()
            .flat_map(|flow| flow.implicits(&mut self.scope))
            .collect();
        self.flows.append(&mut flow_implicits);

        self.flows
            .iter_mut()
            .for_each(|flow| flow.dedup(&mut self.clients));

        self.flows.iter_mut().for_each(|flow| flow.match_rates());

        self.flows
            .iter()
            .for_each(|flow| flow.collect_clients(&mut self.clients));
        self
    }
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut flows: Vec<Flow> = vec![];
        while input.peek(LitInt) {
            flows.push(input.parse::<Flow>()?);
        }

        Ok(Self {
            clients: HashSet::new(),
            flows,
            ..Default::default()
        })
    }
}

impl TryExpand for Model {
    fn try_expand(&self) -> syn::Result<Expanded> {
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
        // println!("{state}");
        let code = quote! {
            use ::gmt_dos_actors::{network::{AddOuput,AddActorOutput,TryIntoInputs,IntoLogs},ArcMutex};
            // ACTORS DEFINITION
            #(#actor_defs)*
            // FLOWS DEFINITION
            #(#flows)*
            // MODEL
            #[allow(unused_variables)]
            let #model = ::gmt_dos_actors::prelude::model!(#(#actors),*).name(#name).flowchart()#state;
        };
        Ok(
            if let Some(_) = self.clients.iter().find(|client| client.is_scope()) {
                self.scope.try_expand()?;
                quote! {
                    let mut monitor = ::gmt_dos_clients_scope::server::Monitor::new();
                    #code
                    monitor.await?;
                }
            } else {
                code
            },
        )
    }
}
