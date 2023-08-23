use std::collections::HashSet;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Ident, LitStr, Token,
};

use crate::{client::SharedClient, Expand, Expanded};

mod flow;
use flow::Flow;

#[derive(Debug, Clone)]
struct KeyParam {
    key: Ident,
    param: LitStr,
}

impl Parse for KeyParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let _ = input.parse::<Token!(=)>()?;
        let param: LitStr = input.parse()?;
        Ok(Self { key, param })
    }
}

#[derive(Debug, Clone)]
struct KeyParams(Vec<KeyParam>);

impl Parse for KeyParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input
                .parse_terminated(KeyParam::parse, Token!(,))?
                .into_iter()
                .collect(),
        ))
    }
}

/// State of the model
///
/// This is state that the model will be into when handed over to the main scope
#[derive(Default, Debug, Clone)]
pub enum ModelState {
    #[default]
    Ready,
    Running,
    Completed,
}
impl From<String> for ModelState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "ready" => Self::Ready,
            "running" => Self::Running,
            "completed" => Self::Completed,
            _ => panic!(r#"expected state "ready", "running" or "completed", found {value}"#),
        }
    }
}

/// Actors model
///
/// A model is a succession of data [Flow]s
#[derive(Debug, Clone)]
pub(super) struct Model {
    pub name: Option<LitStr>,
    pub state: ModelState,
    clients: HashSet<SharedClient>,
    flows: Vec<Flow>,
}
impl Model {
    /// Parse model attributes
    ///
    /// #[model(key = param,...)]
    pub fn attributes(&mut self, attr: Attribute) {
        attr.parse_args::<KeyParams>().ok().map(|kps| {
            kps.0.into_iter().for_each(|kp| {
                let KeyParam { key, param, .. } = kp;
                match key.to_string().as_str() {
                    "name" => {
                        self.name = Some(param);
                    }
                    "state" => self.state = param.value().into(),
                    _ => panic!(r#"expected model attributes "name" or "state", found {key}"#),
                }
            });
        });
    }
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut clients = HashSet::new();
        let mut flows: Vec<Flow> = Punctuated::<Flow, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect();

        flows.iter_mut().for_each(|flow| flow.dedup(&mut clients));
        flows.iter_mut().for_each(|flow| flow.match_rates());
        flows
            .iter()
            .for_each(|flow| flow.collect_clients(&mut clients));

        Ok(Self {
            clients,
            flows,
            name: Default::default(),
            state: Default::default(),
        })
    }
}

impl Expand for Model {
    fn expand(&self) -> Expanded {
        let actor_defs: Vec<_> = self.clients.iter().map(|client| client.expand()).collect();
        let flows: Vec<_> = self.flows.iter().map(|flow| flow.expand()).collect();
        let actors: Vec<_> = self.clients.iter().map(|client| client.actor()).collect();
        let name = self
            .name
            .clone()
            .unwrap_or(LitStr::new("model", Span::call_site()));
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
            let model = ::gmt_dos_actors::prelude::model!(#(#actors),*).name(#name).flowchart()#state;
        }
    }
}
