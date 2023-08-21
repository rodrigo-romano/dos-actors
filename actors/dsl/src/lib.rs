use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute,
};

/* #[proc_macro]
pub fn actorscript0(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as Model);

    let model = input.actors();
    let expanded = quote! {
        use ::gmt_dos_actors::{AddOuput,TryIntoInputs};
        #model
    };
    TokenStream::from(expanded)
} */

#[proc_macro]
pub fn actorscript(input: TokenStream) -> TokenStream {
    let script = parse_macro_input!(input as Script);

    let model = script.expand();
    let expanded = quote! {
        use ::gmt_dos_actors::{AddOuput,TryIntoInputs,ArcMutex};
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

#[derive(Debug, Clone, Default)]
struct Script {
    model: Model,
}

impl Parse for Script {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr = input
            .call(Attribute::parse_outer)?
            .pop()
            .expect("expected the model, log or scope found none");

        dbg!(attr.path().get_ident());

        let model: Model = input.parse()?;
        Ok(Script { model })
    }
}

impl Expand for Script {
    fn expand(&self) -> Expanded {
        let model = self.model.expand();
        quote!(#model)
    }
}
