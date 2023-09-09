/*!
# UID

A derive macro that implements the [UniqueIdentifier] trait.

## Examples

Setting the data type and port # to the default values: `Vec<f64>` and `50_000u32`, respectively:
```
use gmt_dos_clients::interface::UID;

#[derive(UID)]
enum Tag {}
```

The data type and port # are set with:
```
use gmt_dos_clients::interface::UID;

struct Q<T>(std::marker::PhantomData<T>);

enum ID {}

#[derive(UID)]
#[uid(data = Q<ID>, port = 9999)]
enum TU {}
```

An alias is a type that implements the [Read], [Write] or [Size] trait of another type that implements the same traits for the same client:
```
use gmt_dos_clients::interface::{UID, Data, Read, Size, Update, Write};
# struct Q<T>(std::marker::PhantomData<T>);
# enum ID {}
# #[derive(UID)]
# #[uid(data = Q<ID>, port = 9999)]
# enum TU {}

struct Client {}
impl Update for Client {}
impl Write<TU> for Client {
    fn write(&mut self) -> Option<Data<TU>> {
        None
    }
}
impl Read<TU> for Client {
    fn read(&mut self, _data: Data<TU>) {}
}
impl Size<TU> for Client {
    fn len(&self) -> usize {
        1234
    }
}

#[derive(UID)]
#[uid(data = Q<ID>, port = 999)]
#[alias(name = TU, client = Client, traits = Write, Read, Size)]
enum TUT {}
```

[UniqueIdentifier]: https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/trait.UniqueIdentifier.html
[Read]: https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/trait.Read.html
[Write]: https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/trait.Write.html
[Size]: https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/trait.Size.html
*/

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(UID, attributes(uid, alias))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let parser = Parser::new(input).unwrap();
    let expanded = parser.expand(&ident);
    let a = quote! {
        #expanded

    };

    proc_macro::TokenStream::from(a)
}

mod alias;
mod uid;

/// Derive attributes parser
///
/// #[uid(...)]
/// #[alias(...)]
#[derive(Debug, Clone, Default)]
struct Parser {
    pub uid_attrs: uid::Attributes,
    pub alias_attrs: alias::Attributes,
}

impl Parser {
    fn new(input: DeriveInput) -> syn::Result<Parser> {
        let mut parser: Parser = Default::default();
        for attr in input.attrs {
            if attr.path().is_ident("uid") {
                parser.uid_attrs = attr.parse_args()?;
            }
            if attr.path().is_ident("alias") {
                parser.alias_attrs = attr.parse_args()?;
            }
        }
        Ok(parser)
    }
}

type Expanded = proc_macro2::TokenStream;

trait Expand {
    fn expand(&self, ident: &Ident) -> Expanded;
}

impl Expand for Parser {
    fn expand(&self, ident: &Ident) -> Expanded {
        let uid = self.uid_attrs.expand(ident);
        let alias = self.alias_attrs.expand(ident);
        quote! {
            #alias
            #uid
        }
    }
}
