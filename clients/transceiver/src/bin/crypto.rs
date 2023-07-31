use gmt_dos_clients_transceiver::Crypto;

fn main() -> anyhow::Result<()> {
    Crypto::default().generate()?;
    Ok(())
}
