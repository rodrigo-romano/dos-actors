fn main() -> anyhow::Result<()> {
    gmt_fem_code_builder::generate_interface(env!("CARGO_PKG_NAME"))
}
