//! # actorscript

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, Ident, LitStr, Token,
};

#[proc_macro]
pub fn actorscript(input: TokenStream) -> TokenStream {
    let script = parse_macro_input!(input as Script);

    let model = script.expand();
    let expanded = quote! {
        use ::gmt_dos_actors::{AddOuput,TryIntoInputs,ArcMutex,IntoLogs};
        #model
    };
    TokenStream::from(expanded)
}

mod model;
use model::Model;
mod client;

pub(crate) type Expanded = proc_macro2::TokenStream;

pub(crate) trait Expand {
    fn expand(&self) -> Expanded;
}
pub(crate) trait TryExpand {
    fn try_expand(&self) -> Option<Expanded>;
}

#[derive(Debug, Clone)]
struct Script {
    model: Model,
}

impl Parse for Script {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr = input
            .call(Attribute::parse_outer)?
            .pop()
            .expect("expected the model, log or scope found none");

        let mut model: Model = input.parse()?;
        model.attributes(attr);

        let attr = input.call(Attribute::parse_outer).ok();
        dbg!(&attr);

        Ok(Script { model })
    }
}

impl Expand for Script {
    fn expand(&self) -> Expanded {
        let model = self.model.expand();
        quote!(#model)
    }
}

#[derive(Debug, Clone)]
struct KeyParam {
    key: Ident,
    param: LitStr,
}

impl Parse for KeyParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let _ = input.parse::<Token!(=)>()?;
        let param: LitStr = input.parse()?;
        Ok(Self { key, param })
    }
}

#[derive(Debug, Clone)]
struct KeyParams(Vec<KeyParam>);

impl Parse for KeyParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input
                .parse_terminated(KeyParam::parse, Token!(,))?
                .into_iter()
                .collect(),
        ))
    }
}
