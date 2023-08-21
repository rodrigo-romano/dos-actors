use std::collections::HashSet;

use proc_macro2::Span;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token::Bracket,
    GenericParam, Generics, Ident, LitInt, Token, TypeParam,
};

#[derive(Debug, Clone)]
struct Output {
    // output type
    pub name: Ident,
    // ouput options: bootstrap, unbounded
    pub options: Option<Vec<Ident>>,
}

#[derive(Debug, Clone)]
struct Client {
    // client variable
    pub name: Ident,
    // actor variable
    pub actor: Ident,
    // pass client to actor as reference or not
    reference: bool,
    // actor outputs
    output: Option<Output>,
    // actor inputs rate
    pub input_rate: usize,
    // actor output rates
    pub output_rate: usize,
}

impl Client {
    fn quote(&mut self) -> proc_macro2::TokenStream {
        let Self {
            name,
            actor,
            reference,
            input_rate,
            output_rate,
            ..
        } = self.clone();
        let i = LitInt::new(&format!("{input_rate}"), Span::call_site());
        let o = LitInt::new(&format!("{output_rate}"), Span::call_site());
        if reference {
            quote! {
                let #name = #name.into_arcx();
                let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> = Actor::new(#name.clone());
            }
        } else {
            quote! {
                let mut #actor : ::gmt_dos_actors::prelude::Actor<_,#i,#o> = #name.into();
            }
        }
    }
    fn add_output(&self, next_client: &Client) -> Option<proc_macro2::TokenStream> {
        if let Self {
            actor,
            output: Some(output),
            ..
        } = self
        {
            let next_actor = &next_client.actor;
            let Output { name, options } = output;
            options
                .as_ref()
                .map(|options| {
                    quote! {
                        #actor
                        .add_output()
                        #(.#options())*
                        .build::<#name>()
                        .into_input(&mut #next_actor)?;
                    }
                })
                .or_else(|| {
                    Some(quote! {
                        #actor
                        .add_output()
                        .build::<#name>()
                        .into_input(&mut #next_actor)?;
                    })
                })
        } else {
            None
        }
    }
}

impl Parse for Client {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let reference = input.parse::<Token![&]>().is_ok();

        let name: Ident = input.parse()?;

        let output = input
            .peek(Bracket) // looking for an opening bracket: [
            .then(|| {
                let content;
                let _ = bracketed!(content in input); // getting the content within [...,...,...]
                Ok(content
                    .parse_terminated(Ident::parse, Token!(,))?
                    .into_iter()
                    .collect::<Vec<_>>()) // parsing the coma separated content
            })
            .transpose()
            .ok()
            .zip(input.parse::<Generics>().ok().and_then(|generics| {
                // parsing the output type identifer
                generics.params.into_iter().next().map(|g| {
                    let GenericParam::Type(TypeParam { ident, .. }) = g else { todo!() };
                    ident
                })
            }))
            .map(|(options, name)| Output { name, options });

        let actor = if reference {
            Ident::new(&format!("{name}_actor"), Span::call_site())
        } else {
            name.clone()
        };

        Ok(Self {
            name,
            actor,
            output,
            reference,
            input_rate: 0,
            output_rate: 0,
        })
    }
}

#[derive(Debug)]
pub struct Model {
    clients: Vec<Client>,
}

impl Model {
    pub fn actors(&mut self) -> proc_macro2::TokenStream {
        let mut actors = vec![];
        let mut actors_name = HashSet::new();
        let mut model = vec![];
        let mut iter = self.clients.iter_mut().peekable();
        while let Some(output_client) = iter.next() {
            if let Some(input_client) = iter.peek_mut() {
                output_client.output_rate = 1;
                input_client.input_rate = 1;
            }
            if actors_name.insert(output_client.name.clone()) {
                model.push(output_client.actor.clone());
                actors.push(output_client.quote());
            }
        }

        let mut links = vec![];
        let mut iter = self.clients.iter_mut().peekable();
        while let Some(output_client) = iter.next() {
            if let Some(input_client) = iter.peek_mut() {
                output_client
                    .add_output(&input_client)
                    .map_or_else(|| (), |q| links.push(q));
            }
        }

        quote! {
        // ACTORS DEFINITION
        #(#actors)*
        // // ACTORS NETWORK
        #(#links)*
        // MODEL
        let model = ::gmt_dos_actors::prelude::model!(#(#model),*);
        }
    }
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let clients: Vec<_> = input
            .parse_terminated(Client::parse, Token![->])?
            .into_iter()
            .collect();
        Ok(Self { clients })
    }
}
