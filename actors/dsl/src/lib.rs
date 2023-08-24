//! # actorscript
//!
//! A scripting micro-language for [gmt-dos-actors](https://docs.rs/gmt_dos-actors).

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute,
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

/// Source code expansion
pub(crate) trait Expand {
    fn expand(&self) -> Expanded;
}
/// Faillible source code expansion
pub(crate) trait TryExpand {
    fn try_expand(&self) -> Option<Expanded>;
}

/// Script parser
///
/// The script parser holds the code of the actors model
#[derive(Debug, Clone)]
struct Script {
    model: Model,
}

impl Parse for Script {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer).ok();
        let model = input.parse::<Model>()?.attributes(attrs);
        Ok(Script { model })
    }
}

impl Expand for Script {
    fn expand(&self) -> Expanded {
        let model = self.model.expand();
        quote!(#model)
    }
}
