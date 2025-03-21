use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemEnum, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn scopehub(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let hub = input.ident;

    let (scope_ty, signal_ty): (Vec<_>, Vec<_>) = input
        .variants
        .iter()
        .flat_map(|v| {
            v.fields
                .iter()
                .map(|field| (v.ident.clone(), field.ty.clone()))
                .collect::<Vec<(Ident, Type)>>()
        })
        .unzip();

    let idents: Vec<_> = scope_ty
        .iter()
        .enumerate()
        // .map(|field| field.to_string().to_lowercase())
        .map(|(i, _)| format_ident!("scope_{i}"))
        .collect();

    // Build the output, possibly using quasi-quotation
    let scope_hub_server = quote! {
        #[derive(Debug)]
        pub enum ScopeHubError {
            Server(::gmt_dos_clients_scope::server::ServerError)
        }
        impl ::std::fmt::Display for ScopeHubError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "failed to initiate scopes hub")
            }
        }
        impl ::std::error::Error for ScopeHubError {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                match self {
                    Self::Server(source) => Some(source),
                    _ => None
                }
            }
        }
        impl From<::gmt_dos_clients_scope::server::ServerError> for ScopeHubError {
            fn from(e: ::gmt_dos_clients_scope::server::ServerError) -> Self {
                Self::Server(e)
            }
        }
        /// Scopes hub
        pub struct #hub {
            monitor: Option<::gmt_dos_clients_scope::server::Monitor>,
            #(#idents: ::gmt_dos_clients_scope::server::#scope_ty<#signal_ty>),*
        }
        impl #hub {
            /// Creates a new scopes hub instance
            pub fn new() -> Result<Self,ScopeHubError> {
                let mut monitor = ::gmt_dos_clients_scope::server::Monitor::new();
                #(let #idents = ::gmt_dos_clients_scope::server::#scope_ty::<#signal_ty>::builder(&mut monitor).build()?;)*
                Ok(Self {
                    monitor: Some(monitor),
                    #(#idents),*
                })
            }
            /// Closes the scopes hub
            pub async fn close(&mut self) -> Result<(),ScopeHubError> {
                #(self.#idents.end_transmission();)*
                if let Some(monitor) = self.monitor.take() {
                    monitor.join().await.map_err(|e| ::gmt_dos_clients_scope::server::ServerError::Transmitter(e))?;
                }
                Ok(())
            }
        }
        impl ::interface::Update for #hub {}
        #(
        impl ::interface::Read<#signal_ty> for #hub {
            fn read(&mut self, data: ::interface::Data<#signal_ty>)  {
                <_ as ::interface::Read<#signal_ty>>::read(&mut self.#idents, data);
            }
        }
        )*
        impl ::std::future::IntoFuture for &mut #hub {
            type Output = <::gmt_dos_clients_scope::server::Monitor as ::std::future::IntoFuture>::Output;
            type IntoFuture = <::gmt_dos_clients_scope::server::Monitor as ::std::future::IntoFuture>::IntoFuture;
            fn into_future(self) -> Self::IntoFuture {
                #(self.#idents.end_transmission();)*
                self.monitor.take().unwrap().into_future()
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(scope_hub_server)
}
