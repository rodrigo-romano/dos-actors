/*!
# UID

A derive macro that implements the [UniqueIdentifier] trait.

## Examples

Setting the data type and port # to the default values: `Vec<f64>` and `50_000u32`, respectively:
```
use interface::UID;

#[derive(UID)]
enum Tag {}
```

The data type and port # are set with:
```
use interface::UID;

struct Q<T>(std::marker::PhantomData<T>);

enum ID {}

#[derive(UID)]
#[uid(data = Q<ID>, port = 9999)]
enum TU {}
```

An alias is a type that implements the [Read], [Write] or [Size] trait of another type that implements the same traits for the same client:
```
use interface::{UID, Data, Read, Size, Update, Write};
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
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UID, attributes(uid, alias))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    Parser::new(&input)
        .map_or_else(syn::Error::into_compile_error, |parser| {
            if let Some(alias_attrs) = parser.alias_attrs {
                let aliases: Vec<_> = alias_attrs
                    .iter()
                    .map(|alias| alias.expand(&input))
                    .collect();
                quote!(#(#aliases)*)
            } else {
                parser.uid_attrs.expand(&input)
            }
        })
        .into()
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
    pub alias_attrs: Option<Vec<alias::Attributes>>,
}

impl Parser {
    fn new(input: &DeriveInput) -> syn::Result<Parser> {
        let mut parser: Parser = Default::default();
        for attr in &input.attrs {
            if attr.path().is_ident("uid") {
                parser.uid_attrs = attr.parse_args()?;
            }
            if attr.path().is_ident("alias") {
                parser
                    .alias_attrs
                    .get_or_insert(vec![])
                    .push(attr.parse_args()?);
            }
        }
        parser
            .alias_attrs
            .iter_mut()
            .flatten()
            .skip(1)
            .for_each(|alias| {
                alias.skip_uid = true;
            });
        Ok(parser)
    }
}

type Expanded = proc_macro2::TokenStream;

trait Expand {
    fn expand(&self, input: &DeriveInput) -> Expanded;
}
