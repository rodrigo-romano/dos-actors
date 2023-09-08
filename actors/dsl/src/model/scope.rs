use proc_macro2::Span;
use quote::quote;
use syn::{Ident, LitStr};

use crate::Expand;

/// Parameters to expand `gmt_dos-clients_scope`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Scope {
    pub server: String,
    pub client: String,
    pub signals: Vec<Ident>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            server: String::from("127.0.0.1"),
            client: String::from("127.0.0.1:0"),
            signals: Vec::new(),
        }
    }
}
impl Scope{
pub fn lit_server(&self) -> LitStr {
    LitStr::new(self.server.as_str(), Span::call_site())
}
pub fn lit_client(&self) -> LitStr {
    LitStr::new(self.client.as_str(), Span::call_site())
}}
impl Expand for Scope {
    fn expand(&self) -> crate::Expanded {
        let signals: Vec<_> = self.signals
            .iter()
            .map(|signal| 
                quote!(.signal::<#signal>(<#signal as ::gmt_dos_clients::interface::UniqueIdentifier>::PORT)?))
            .collect();
        let (server,client) = (self.lit_server(),self.lit_client());
        quote! {
            ::gmt_dos_clients_scope::client::Scope::new(#server, #client)
                #(#signals)*
                .show();
        }
    }
}
