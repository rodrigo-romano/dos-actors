use std::{cell::RefCell, hash::Hash, ops::Deref, rc::Rc};

use proc_macro2::{Literal, Span};
use quote::quote;
use syn::{Ident, LitInt};

use crate::{Expand, Expanded};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ClientKind {
    MainScope,
    Sampler,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    // client variable
    pub name: Ident,
    // actor variable
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
            input_rate,
            output_rate,
            kind,
        } = self.clone();
        let i = LitInt::new(&format!("{input_rate}"), Span::call_site());
        let o = LitInt::new(&format!("{output_rate}"), Span::call_site());
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
                ClientKind::Sampler => quote! {
                    let mut #actor: ::gmt_dos_actors::prelude::Actor::<_,#i,#o> = ::gmt_dos_clients::Sampler::default().into();
                },
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct SharedClient(Rc<RefCell<Client>>);
impl SharedClient {
    pub fn new(name: Ident, actor: Ident, reference: bool) -> Self {
        Self(Rc::new(RefCell::new(Client {
            name,
            actor,
            reference,
            input_rate: 0,
            output_rate: 0,
            kind: ClientKind::MainScope,
        })))
    }
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
