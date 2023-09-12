use std::{fs::{create_dir_all, File}, path::PathBuf};

use proc_macro2::Span;
use quote::quote;
use syn::{Ident, LitStr};

use crate::{Expand, TryExpand, Expanded};

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

use std::io::Write;

impl TryExpand for Scope {
    fn try_expand(&self) -> syn::Result<Expanded>  {
        let root: PathBuf = std::env::var("CARGO_BIN_NAME")
            .map_or(Default::default(), Into::into);
        let bin: PathBuf = root.join("src").join("bin");
        create_dir_all(&bin).map_err(|e| syn::Error::new(Span::call_site(),e))?;
        for signal in &self.signals {
            let scope_name = format!("scope-{}",signal);
            let path = bin.join(scope_name).with_extension("rs");
            if path.is_file() {
                println!("WARNING: scope {} already exists",path.display());
                continue;
            }
            let mut scope = File::create(path).map_err(|e| syn::Error::new(Span::call_site(),e))?;
            write!(&mut scope,r#"
#![allow(non_snake_case)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let port =  <{signal} as ::gmt_dos_clients::interface::UniqueIdentifier>::PORT;
    dbg!(&port);
    ::gmt_dos_clients_scope::client::Scope::new("{server}", "{client}")
    .signal::<{signal}>(port)?
    .show();
    Ok(())
}}
            "#,
        server = self.server,
        client=self.client,
        signal=signal).map_err(|e| syn::Error::new(Span::call_site(),e))?;
        }
        Ok(quote!())
    }
}