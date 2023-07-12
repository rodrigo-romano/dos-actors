#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ngao_opm::ngao_opm().await?;
    Ok(())
}
