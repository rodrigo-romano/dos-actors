use std::{cell::RefCell, hash::Hash, ops::Deref, rc::Rc};

use proc_macro2::{Literal, Span};
use quote::quote;
use syn::{Ident, LitInt, LitStr};

use crate::{Expand, Expanded};

const LOG_BUFFER_SIZE: usize = 1_000;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ClientKind {
    MainScope,
    Sampler,
    Logger,
}

/// Actor client
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    // client variable name
    pub name: Ident,
    // actor variable name
    pub actor: Ident,
    // pass client to actor as reference or not
    pub reference: bool,
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
}
impl Expand for Client {
    fn expand(&self) -> Expanded {
        let Self {
            name,
            actor,
            reference,
            kind,
            ..
        } = self.clone();
        let (i, o) = (self.lit_input_rate(), self.lit_output_rate());
        if reference {
            quote! {
                let #name = #name.into_arcx();
                let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> =
                    ::gmt_dos_actors::prelude::Actor::new(#name.clone());
            }
        } else {
            match kind {
                ClientKind::MainScope => quote! {
                    let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> = #name.into();
                },
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
                ClientKind::Logger => {
                    let filename = LitStr::new(actor.to_string().as_str(), Span::call_site());
                    let buffer_size = LitInt::new(&format!("{LOG_BUFFER_SIZE}"), Span::call_site());
                    quote! {
                        let mut #actor: ::gmt_dos_actors::prelude::Actor::<_,#i,#o> =
                            (::gmt_dos_clients_arrow::Arrow::builder(#buffer_size).filename(#filename).build(),#filename).into();
                    }
                }
            }
        }
    }
}

/// Shared client with interior mutability
#[derive(Debug, Clone, Eq)]
pub struct SharedClient(Rc<RefCell<Client>>);
impl SharedClient {
    /// Creates a new client from the main scope
    pub fn new(name: Ident, reference: bool) -> Self {
        let actor = if reference {
            Ident::new(&format!("{name}_actor"), Span::call_site())
        } else {
            name.clone()
        };
        Self(Rc::new(RefCell::new(Client {
            name,
            actor,
            reference,
            input_rate: 0,
            output_rate: 0,
            kind: ClientKind::MainScope,
        })))
    }
    /// Creates a sampler client from [gmt_dos-clients::Sampler](https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/struct.Sampler.html)
    pub fn sampler(actor: Ident, name: Ident, output_rate: usize, input_rate: usize) -> Self {
        let sampler = Ident::new(
            &format!("sampler_{}_{}_{}r{}", actor, name, input_rate, output_rate),
            Span::call_site(),
        );
        Self(Rc::new(RefCell::new(Client {
            name: sampler.clone(),
            actor: sampler,
            reference: false,
            input_rate,
            output_rate,
            kind: ClientKind::Sampler,
        })))
    }
    /// Creates a sampler client from [gmt_dos-clients_arrow](https://docs.rs/gmt_dos-clients_arrow)
    pub fn logger(input_rate: usize) -> Self {
        let logger = Ident::new(&format!("data_{}", input_rate), Span::call_site());
        Self(Rc::new(RefCell::new(Client {
            name: logger.clone(),
            actor: logger,
            reference: false,
            input_rate,
            output_rate: 0,
            kind: ClientKind::Logger,
        })))
    }
    // pub fn name(&self) -> Ident {
    //     self.0.borrow().name.clone()
    // }
    pub fn actor(&self) -> Ident {
        self.0.borrow().actor.clone()
    }
}

impl Expand for SharedClient {
    fn expand(&self) -> Expanded {
        self.borrow().expand()
    }
}
impl Deref for SharedClient {
    type Target = RefCell<Client>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl Hash for SharedClient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.borrow().hash(state);
    }
}
impl PartialEq for SharedClient {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
