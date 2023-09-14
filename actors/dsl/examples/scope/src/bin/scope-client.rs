/*
Model MODEL [ready] :
  1: cfd_loads[cfdm1windloads]
  1: cfd_loads[cfdm1windloads] -> scope_cfdm1windloads
  1: cfd_loads[cfdm1windloads] -> data_1
 */
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ::gmt_dos_clients_scope::client::Scope::new("127.0.0.1", "127.0.0.1:0")
        .signal::<
            gmt_dos_clients_io::cfd_wind_loads::CFDM1WindLoads,
        >(
            <gmt_dos_clients_io::cfd_wind_loads::CFDM1WindLoads as ::gmt_dos_clients::interface::UniqueIdentifier>::PORT,
        )?
        .show();
    Ok(())
}

