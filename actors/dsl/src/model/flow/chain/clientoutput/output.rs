use std::{collections::HashSet, fmt::Display};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseBuffer, ParseStream},
    token::Bracket,
    Ident, PathSegment, Token, Type, TypePath,
};

use crate::client::SharedClient;

/// Actor ouput
#[derive(Debug, Clone)]
pub struct Output {
    // output type
    pub ty: Type,
    pub name: String,
    // ouput options: bootstrap, unbounded
    pub options: Option<Vec<Ident>>,
    // need a rate transition
    pub rate_transition: Option<SharedClient>,
    // need a scope
    pub scope: bool,
    pub logging: bool,
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.name)
    }
}

impl Output {
    /// Creates a new output
    pub fn new(ty: Type) -> syn::Result<Self> {
        if let Type::Path(TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = &ty
        {
            segments
                .iter()
                .rev()
                .find_map(|segment| match segment {
                    PathSegment { ident, arguments } if arguments.is_none() => Some(ident.clone()),
                    _ => None,
                })
                .ok_or(syn::Error::new(
                    Span::call_site(),
                    &format!("no valid ident for Output of type {:?}", &ty),
                ))
                .map(|ident| ident.to_string().to_lowercase())
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                &format!("expected Output Type variant Path found {:?}", &ty),
            ))
        }
        .map(|name| Self {
            ty,
            name,
            // generics,
            options: None,
            rate_transition: None,
            scope: false,
            logging: false,
        })
    }
    pub fn expand_name(&self) -> TokenStream {
        let ty = &self.ty;
        quote!(#ty)
    }
    /// Clone and collect any sampler clients
    pub fn collect(&self, clients: &mut HashSet<SharedClient>) {
        self.rate_transition
            .as_ref()
            .map(|client| clients.insert(client.clone()));
    }
    /// Add a rate transition sampler client
    pub fn add_rate_transition(&mut self, output_rate: usize, input_rate: usize) {
        self.rate_transition = Some(SharedClient::sampler(
            self.name.as_str(),
            output_rate,
            input_rate,
        ));
    }
    pub fn add_option(&mut self, option: &str) {
        self.options
            .get_or_insert(vec![])
            .push(Ident::new(option, Span::call_site()));
    }
    pub fn add_logging(&mut self) {
        self.logging = true;
    }
    pub fn add_scope(&mut self) {
        self.scope = true;
    }
}
impl<'a> TryFrom<ParseBuffer<'a>> for Output {
    type Error = syn::parse::Error;

    fn try_from(content: ParseBuffer<'a>) -> Result<Self, Self::Error> {
        content.parse::<Type>().and_then(|ty| Output::new(ty))
    }
}
impl Parse for Output {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // looking for an output name within brackets i.e. client[output_name]
        input
            .peek(Bracket)
            .then(|| {
                let content;
                let _ = bracketed!(content in input);
                let mut output = Output::try_from(content)?;
                // checking out for output options either !, .. or $ ,
                // or any combination of the 3 after the output i.e.
                // client[output_name]!$
                loop {
                    match (
                        input.peek(Token![!]),
                        input.peek(Token![$]),
                        input.peek(Token![..]),
                        input.peek(Token![~]),
                    ) {
                        (true, false, false, false) => {
                            input
                                .parse::<Token![!]>()
                                .map(|_| output.add_option("bootstrap"))?;
                        }
                        (false, true, false, false) => {
                            input.parse::<Token![$]>().map(|_| output.add_logging())?;
                        }
                        (false, false, true, false) => {
                            input
                                .parse::<Token![..]>()
                                .map(|_| output.add_option("unbounded"))?;
                        }
                        (false, false, false, true) => {
                            input.parse::<Token![~]>().map(|_| output.add_scope())?;
                        }
                        (false, false, false, false) => break,
                        _ => unimplemented!(),
                    }
                }
                Ok(output)
            })
            .ok_or(syn::Error::new(input.span(), "no output given "))
            .and_then(|maybe_output| maybe_output)
    }
}
