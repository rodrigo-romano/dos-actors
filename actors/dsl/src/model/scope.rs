use proc_macro2::Span;
use quote::quote;
use syn::{LitStr, Type};

use crate::{Expand, Expanded, TryExpand};

// Scope signal
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopeSignal {
    pub ty: Type,
    pub name: String,
}

/// Parameters to expand `gmt_dos-clients_scope`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Scope {
    pub server: String,
    pub client: String,
    pub signals: Vec<ScopeSignal>,
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
impl Scope {
    pub fn lit_server(&self) -> LitStr {
        LitStr::new(self.server.as_str(), Span::call_site())
    }
    pub fn lit_client(&self) -> LitStr {
        LitStr::new(self.client.as_str(), Span::call_site())
    }
    pub fn _expand(&self) -> (Vec<Expanded>, Vec<Expanded>) {
        let (server, client) = (self.lit_server(), self.lit_client());
        let mut tokens = vec![];
        let n_scope = self.signals.len();
        let mut names = vec![];
        for signal in self.signals.iter() {
            let ScopeSignal { ty, name } = signal;
            let na = LitStr::new(name.as_str(), Span::call_site());

            let scope = quote! {
                ::gmt_dos_clients_scope::client::Scope::new(#server, #client)
                    .signal::<#ty>(<#ty as ::interface::UniqueIdentifier>::PORT)?
                    .show()
            };

            names.push(quote!(#na));

            tokens.push(if n_scope > 1 {
                quote! {
                #na => #scope,
                }
            } else {
                quote!(#scope;)
            })
        }
        (tokens, names)
    }
}
impl Expand for Scope {
    fn expand(&self) -> crate::Expanded {
        let signals: Vec<_> = self.signals
            .iter()
            .map(|signal| {
                let ty = &signal.ty;
                quote!(.signal::<#ty>(<#ty as ::gmt_dos_clients::interface::UniqueIdentifier>::PORT)?)
            })
            .collect();
        let (server, client) = (self.lit_server(), self.lit_client());
        quote! {
            ::gmt_dos_clients_scope::client::Scope::new(#server, #client)
                #(#signals)*
                .show();
        }
    }
}

// use std::io::Write;

impl TryExpand for Scope {
    fn try_expand(&self) -> syn::Result<Expanded> {
        /*         let (scopes, names) = self.expand();
        let clients = if names.len() > 1 {
            quote! {
                let mut args = std::env::args();
                let msg = format!("expected one argument of {:?}, found none",(#(#names),*));
                args.next();
                match args.next().as_ref().expect(&msg).as_str()  {
                    #(#scopes)*
                    _ => unimplemented!("{}",&msg)
                }
            }
        } else {
            quote! {#(#scopes)*}
        };
        let main = quote! {

            #[tokio::main]
            async fn main() -> anyhow::Result<()> {
                #clients
                Ok(())
            }
        };
        let main = syn::parse_file(main.to_string().as_str())?;
        let main = prettyplease::unparse(&main);
        writeln!(std::io::stdout(), "{}", main).map_err(|e| {
            syn::Error::new(
                Span::call_site(),
                &format!("failed to write scope client to sdout due to:\n{}", e),
            )
        })?; */

        let signals: Vec<_> = self
            .signals
            .iter()
            .map(|signal| {
                let ScopeSignal { ty, .. } = signal;
                quote!(.signal::<#ty>()?)
            })
            .collect();

        let (server, client) = (self.lit_server(), self.lit_client());

        Ok(quote! {
            ::gmt_dos_clients_scope_client::Scope::new(#server, #client)
            #(#signals)*
            .show();
        })
    }
}
