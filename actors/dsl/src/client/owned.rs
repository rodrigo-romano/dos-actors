use std::fmt::Display;

use proc_macro2::{Literal, Span};
use quote::quote;
use syn::{Expr, Ident, LitInt, LitStr};

use crate::{model::ScopeSignal, Expand, Expanded};

const LOG_BUFFER_SIZE: usize = 1_000;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ClientKind {
    MainScope,
    Sampler,
    Logger(Ident, Option<Expr>),
    Scope { server: LitStr, signal: ScopeSignal },
    SubSystem,
}
impl ClientKind {
    pub fn is_scope(&self) -> bool {
        match self {
            Self::Scope { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reference {
    Value,
    Reference,
    Pointer,
}

/// Actor client
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    // client variable name
    pub name: Ident,
    // actor variable name
    pub actor: Ident,
    // pass client to actor as reference or not
    pub reference: Reference,
    // actor label
    pub label: Option<LitStr>,
    // actor inputs rate
    pub input_rate: usize,
    // actor output rates
    pub output_rate: usize,
    // client type
    pub kind: ClientKind,
}
impl Client {
    pub fn lit_output_rate(&self) -> LitInt {
        Literal::usize_unsuffixed(self.output_rate).into()
    }
    pub fn lit_input_rate(&self) -> LitInt {
        Literal::usize_unsuffixed(self.input_rate).into()
    }
    pub fn into_input(&self) -> Expanded {
        let actor = &self.actor;
        match &self.kind {
            ClientKind::Logger(_, None) => quote!(.log(&mut #actor).await?;),
            ClientKind::Logger(_, Some(size)) => quote!(.logn(&mut #actor, #size).await?;),
            _ => quote!(.into_input(&mut #actor)?;),
        }
    }
    pub fn is_scope(&self) -> bool {
        self.kind.is_scope()
    }
}
impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ClientKind::MainScope | ClientKind::SubSystem => write!(
                f,
                "main client: {} into actor: {} with rates: {} input & {} output",
                self.name, self.actor, self.input_rate, self.output_rate
            ),
            ClientKind::Sampler => write!(
                f,
                "Sampler client: {} into actor: {} with rates: {} input & {} output",
                self.name, self.actor, self.input_rate, self.output_rate
            ),
            ClientKind::Logger(..) => write!(
                f,
                "Arrow client: {} into actor: {} with rates: {} input & {} output",
                self.name, self.actor, self.input_rate, self.output_rate
            ),
            ClientKind::Scope { .. } => write!(
                f,
                "Scope client: {} into actor: {} with rates: {} input & {} output",
                self.name, self.actor, self.input_rate, self.output_rate
            ),
        }
    }
}
impl Expand for Client {
    fn expand(&self) -> Expanded {
        let Self {
            name,
            actor,
            label,
            reference,
            kind,
            ..
        } = self;
        let (i, o) = (self.lit_input_rate(), self.lit_output_rate());
        match kind {
            ClientKind::MainScope => match (reference, label.as_ref()) {
                (Reference::Reference, None) => quote! {
                    let #name = #name.into_arcx();
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> =
                        ::gmt_dos_actors::prelude::Actor::new(#name.clone());
                },
                (Reference::Reference, Some(label)) => quote! {
                    let #name = #name.into_arcx();
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> =
                        ::gmt_dos_actors::prelude::Actor::new(#name.clone()).name(#label);
                },
                (Reference::Value, None) => quote! {
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> = #name.into();
                },
                (Reference::Value, Some(label)) => quote! {
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> = (#name,#label).into();
                },
                (Reference::Pointer, None) => quote! {
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> =
                        ::gmt_dos_actors::prelude::Actor::new(#name.clone());
                },
                (Reference::Pointer, Some(label)) => quote! {
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> =
                        ::gmt_dos_actors::prelude::Actor::new(#name.clone()).name(#label);
                },
            },
            ClientKind::SubSystem => quote!(),
            ClientKind::Sampler => {
                let sampler_type = LitStr::new(
                    if self.input_rate < self.output_rate {
                        "downsampling"
                    } else {
                        "upsampling"
                    },
                    Span::call_site(),
                );
                quote! {
                    let mut #actor: ::gmt_dos_actors::prelude::Actor::<_,#i,#o> =
                        (::gmt_dos_clients::Sampler::default(),format!("{}\n{}:{}",#sampler_type,#i,#o)).into();
                }
            }
            ClientKind::Logger(name, _) => {
                let filename = LitStr::new(&format!("{name}-{actor}"), Span::call_site());
                let buffer_size = LitInt::new(&format!("{LOG_BUFFER_SIZE}"), Span::call_site());
                quote! {
                    let mut #name = ::gmt_dos_clients_arrow::Arrow::builder(#buffer_size).filename(#filename).build().into_arcx();
                    let mut #actor: ::gmt_dos_actors::prelude::Actor::<_,#i,#o> =
                        ::gmt_dos_actors::prelude::Actor::new(#name.clone()).name(#filename);
                }
            }
            ClientKind::Scope {
                server,
                signal: ScopeSignal { ty, .. },
            } => {
                quote! {
                    let socket  = format!("{}:{}",
                        #server,
                        <#ty as ::interface::UniqueIdentifier>::PORT);
                    let mut #actor : ::gmt_dos_actors::prelude::Actor::<_,#i,#o> =
                        ::gmt_dos_clients_scope::server::Scope::<#ty>::builder(socket, &mut monitor)
                            .build()?
                            .into();
                }
            }
        }
    }
}
