use std::{cell::RefCell, fmt::Display, hash::Hash, ops::Deref, rc::Rc};

use proc_macro2::Span;
use quote::format_ident;
use syn::{Ident, LitStr};

use crate::{model::Scope, Expand, Expanded};

use super::{Client, ClientKind};

/// Shared client with interior mutability
#[derive(Debug, Clone, Eq)]
pub struct SharedClient(Rc<RefCell<Client>>);
impl SharedClient {
    /// Creates a new client from the main scope
    pub fn new(name: Ident, reference: bool, label: Option<LitStr>) -> Self {
        let actor = if reference {
            Ident::new(&format!("{name}_actor"), Span::call_site())
        } else {
            name.clone()
        };
        Self(Rc::new(RefCell::new(Client {
            name,
            actor,
            label,
            reference,
            input_rate: 0,
            output_rate: 0,
            kind: ClientKind::MainScope,
        })))
    }
    /// Creates a sampler client from [gmt_dos-clients::Sampler](https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/struct.Sampler.html)
    pub fn sampler(name: Ident, output_rate: usize, input_rate: usize) -> Self {
        let sampler = format_ident!(
            "_{}_{}_{}_",
            input_rate,
            name.to_string().to_lowercase(),
            output_rate
        );
        Self(Rc::new(RefCell::new(Client {
            name: sampler.clone(),
            actor: sampler,
            label: None,
            reference: false,
            input_rate,
            output_rate,
            kind: ClientKind::Sampler,
        })))
    }
    /// Creates a sampler client from [gmt_dos-clients_arrow](https://docs.rs/gmt_dos-clients_arrow)
    pub fn logger(input_rate: usize) -> Self {
        let name = format_ident!("logging_{}", input_rate);
        let actor = format_ident!("data_{}", input_rate);
        Self(Rc::new(RefCell::new(Client {
            name,
            actor,
            label: None,
            reference: false,
            input_rate,
            output_rate: 0,
            kind: ClientKind::Logger,
        })))
    }
    /// Creates a scope client from [gmt_dos-clients_scope](https://docs.rs/gmt_dos-clients_scope)
    pub fn scope(output_name: &Ident, input_rate: usize, scope: &mut Scope) -> Self {
        // let name = Ident::new(&format!("scope_{}", output_name), output_name.span());
        let name = output_name.clone();
        scope.signals.push(name.clone());
        let actor = format_ident!("scope_{}", output_name.to_string().to_lowercase());
        Self(Rc::new(RefCell::new(Client {
            name,
            actor,
            label: None,
            reference: false,
            input_rate,
            output_rate: 0,
            kind: ClientKind::Scope(scope.lit_server()),
        })))
    }
    // pub fn name(&self) -> Ident {
    //     self.0.borrow().name.clone()
    // }
    pub fn actor(&self) -> Ident {
        self.borrow().actor.clone()
    }
    pub fn is_scope(&self) -> bool {
        self.borrow().is_scope()
    }
}
impl Display for SharedClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
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