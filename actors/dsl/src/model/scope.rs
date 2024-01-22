use quote::quote;
use syn::Type;

use crate::{Expanded, TryExpand};

// Scope signal
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopeSignal {
    pub ty: Type,
    pub name: String,
}

/// Parameters to expand `gmt_dos-clients_scope`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Scope {
    pub signals: Vec<ScopeSignal>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            signals: Vec::new(),
        }
    }
}

impl TryExpand for Scope {
    fn try_expand(&self) -> syn::Result<Expanded> {
        let signals: Vec<_> = self
            .signals
            .iter()
            .map(|signal| {
                let ScopeSignal { ty, .. } = signal;
                quote!(.signal::<#ty>()?)
            })
            .collect();

        Ok(quote! {
            ::gmt_dos_clients_scope_client::Scope::new()
            #(#signals)*
            .show();
        })
    }
}
