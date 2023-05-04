# ARROW

The crate `Arrow` is a data logger for the outputs of the actors.
The data is recorded in the [Apache Arrow] format.
Compared to the `Logging` client, data with different data types can be aggregated into the same [Apache Arrow] record.
The data is automatically saved to a [Parquet] file.
For proper usage, consults the documentation.

[Apache Arrow]: https://docs.rs/arrow
[Parquet]: https://docs.rs/parquet

* DOS Client

|||||
|-|-|-|-|
|`gmt_dos-clients_arrow`| [crates.io](https://crates.io/crates/gmt_dos-clients_arrow) | [docs.rs](https://docs.rs/gmt_dos-clients_arrow) | [github](https://github.com/rconan/dos-actors/tree/main/clients/arrow) |
