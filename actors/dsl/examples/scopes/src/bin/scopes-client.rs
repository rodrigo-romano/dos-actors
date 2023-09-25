/*
Model MODEL [ready] :
  1: cfd_loads[cfdm1windloads]
  1: cfd_loads[cfdm2windloads]
  1: cfd_loads[cfdm1windloads] -> scope_cfdm1windloads
  1: cfd_loads[cfdm2windloads] -> scope_cfdm2windloads
 */
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let msg = format!(
        "expected arguments in {:?}, found none",
        ("cfdm1windloads", "cfdm2windloads")
    );
    args.next();
    match args.next().as_ref().expect(&msg).as_str() {
        "cfdm1windloads" => {
            ::gmt_dos_clients_scope::client::Scope::new("127.0.0.1", "127.0.0.1:0")
                .signal::<
                    gmt_dos_clients_io::cfd_wind_loads::CFDM1WindLoads,
                >(
                    <gmt_dos_clients_io::cfd_wind_loads::CFDM1WindLoads as ::interface::UniqueIdentifier>::PORT,
                )?
                .show()
        }
        "cfdm2windloads" => {
            ::gmt_dos_clients_scope::client::Scope::new("127.0.0.1", "127.0.0.1:0")
                .signal::<
                    gmt_dos_clients_io::cfd_wind_loads::CFDM2WindLoads,
                >(
                    <gmt_dos_clients_io::cfd_wind_loads::CFDM2WindLoads as ::interface::UniqueIdentifier>::PORT,
                )?
                .show()
        }
        _ => unimplemented!(),
    }
    Ok(())
}
