use proc_macro::TokenStream;
use quote::quote;
use syn::{
    bracketed, parenthesized, parse::Parse, parse_macro_input, Ident, LitInt, LitStr, Token,
};

#[proc_macro]
pub fn scope(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Scope);
    let scope = input.scope();
    let variables = input.variables();
    let signals = input.signals();
    let expanded = quote! {
        #(#variables)*
        #scope
        #(#signals)*
        .show();
    };
    TokenStream::from(expanded)
}

struct Signal {
    ident: Ident,
    port: LitInt,
}
impl Signal {
    fn variable(&self) -> proc_macro2::TokenStream {
        let Signal { ident, .. } = self;
        quote!(
            #[derive(::gmt_dos_clients::interface::UID)]
            #[uid(data = "f64")]
            pub enum #ident {}
        )
    }
    fn signal(&self) -> proc_macro2::TokenStream {
        let Signal { ident, port } = self;
        quote! {
            .signal::<#ident>(#port)?
        }
    }
}

impl Parse for Signal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let port: LitInt = input.parse()?;
        Ok(Self { ident, port })
    }
}

struct ParenthesizedSignal(Signal);

impl Parse for ParenthesizedSignal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let signal: Signal = content.parse()?;
        Ok(Self(signal))
    }
}

// #[derive(Debug)]
struct Scope {
    server_address: LitStr,
    client_address: LitStr,
    signals: Vec<Signal>,
}

impl Scope {
    pub fn scope(&self) -> proc_macro2::TokenStream {
        let Self {
            server_address,
            client_address,
            ..
        } = self;
        quote! {
            ::gmt_dos_clients_scope::Scope::new(#server_address, #client_address)
        }
    }
    pub fn signals(&self) -> Vec<proc_macro2::TokenStream> {
        self.signals.iter().map(|signal| signal.signal()).collect()
    }
    pub fn variables(&self) -> Vec<proc_macro2::TokenStream> {
        self.signals
            .iter()
            .map(|signal| signal.variable())
            .collect()
    }
}

impl Parse for Scope {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let server_address: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let client_address: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        let _ = bracketed!(content in input);
        let signals: Vec<_> = content
            .parse_terminated(ParenthesizedSignal::parse, Token![,])?
            .into_iter()
            .map(|x| x.0)
            .collect();

        Ok(Self {
            server_address,
            client_address,
            signals,
        })
    }
}
