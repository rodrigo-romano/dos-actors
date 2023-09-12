use proc_macro::TokenStream;
use quote::quote;
use syn::{
    bracketed, parenthesized, parse::Parse, parse_macro_input, Expr, Ident, LitInt, LitStr, Token,
};

/**
Signal plotting scope

## Example

```ignore
use gmt_dos_clients_scope::client;

let server_ip = "127.0.0.1";
let server_port = 5001;
let client_address = "127.0.0.1:0";

client::scope!(server_ip, client_address, [(Signal, server_port)]);
```

*/
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

/**
Image display scope

## Example

```ignore
use gmt_dos_clients_scope::client;

let server_ip = "127.0.0.1";
let server_port = 5001;
let client_address = "127.0.0.1:0";

client::shot!(server_ip, client_address, [(Signal, server_port)]);
```
*/
#[proc_macro]
pub fn shot(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Scope);
    let scope = input.shot();
    let variables = input.images();
    let signals = input.signals();
    let expanded = quote! {
        #(#variables)*
        #scope
        #(#signals)*
        .show();
    };
    TokenStream::from(expanded)
}

/**
GMT scope

## Example

```ignore
use gmt_dos_clients_scope::client;

let server_ip = "127.0.0.1";
let server_port = 5001;
let client_address = "127.0.0.1:0";

client::gmt_scope!(server_ip, client_address, [(GmtWavefront, server_port)]);
```
*/
#[proc_macro]
pub fn gmt_shot(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Scope);
    let scope = input.gmt_shot();
    let variables = input.gmt_images();
    let signals = input.signals();
    let expanded = quote! {
        #(#variables)*
        #scope
        #(#signals)*
        .show();
    };
    TokenStream::from(expanded)
}

enum Port {
    LitInt(LitInt),
    Expr(Expr),
}
impl From<LitInt> for Port {
    fn from(value: LitInt) -> Self {
        Self::LitInt(value)
    }
}
impl From<Expr> for Port {
    fn from(value: Expr) -> Self {
        Self::Expr(value)
    }
}
impl Port {
    pub fn port(&self) -> proc_macro2::TokenStream {
        match self {
            Port::LitInt(value) => quote!(#value),
            Port::Expr(value) => quote!(#value),
        }
    }
}

struct Signal {
    ident: Ident,
    port: Port,
}
impl Signal {
    fn variable(&self) -> proc_macro2::TokenStream {
        let Signal { ident, .. } = self;
        quote!(
            #[derive(::gmt_dos_clients::interface::UID)]
            #[uid(data = f64)]
            pub enum #ident {}
        )
    }
    fn image(&self) -> proc_macro2::TokenStream {
        let Signal { ident, .. } = self;
        quote!(
            #[derive(::gmt_dos_clients::interface::UID)]
            pub enum #ident {}
        )
    }
    fn gmt_image(&self) -> proc_macro2::TokenStream {
        let Signal { ident, .. } = self;
        quote!(
            #[derive(::gmt_dos_clients::interface::UID)]
            #[uid(data = (Vec<f64>,Vec<bool>))]
            pub enum #ident {}
        )
    }
    fn signal(&self) -> proc_macro2::TokenStream {
        let Signal { ident, port } = self;
        let port = port.port();
        quote! {
            .signal::<#ident>(#port)?
        }
    }
}

impl Parse for Signal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let port: Port = if let Ok(port) = input.parse::<LitInt>() {
            Ok(port.into())
        } else {
            input.parse::<Expr>().map(|port| Port::from(port))
        }?;
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
            ::gmt_dos_clients_scope::client::Scope::new(#server_address, #client_address)
        }
    }
    pub fn shot(&self) -> proc_macro2::TokenStream {
        let Self {
            server_address,
            client_address,
            ..
        } = self;
        quote! {
            ::gmt_dos_clients_scope::client::Shot::new(#server_address, #client_address)
        }
    }
    pub fn gmt_shot(&self) -> proc_macro2::TokenStream {
        let Self {
            server_address,
            client_address,
            ..
        } = self;
        quote! {
            ::gmt_dos_clients_scope::client::GmtShot::new(#server_address, #client_address)
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
    pub fn images(&self) -> Vec<proc_macro2::TokenStream> {
        self.signals.iter().map(|signal| signal.image()).collect()
    }
    pub fn gmt_images(&self) -> Vec<proc_macro2::TokenStream> {
        self.signals
            .iter()
            .map(|signal| signal.gmt_image())
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
