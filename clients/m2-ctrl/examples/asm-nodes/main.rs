use gmt_dos_clients_fem::fem_io;
use gmt_dos_clients_m2_ctrl::nodes;
use gmt_fem::FEM;

fn main() -> anyhow::Result<()> {
    let fem = FEM::from_env()?;
    println!("{fem}");

    let mut nodes = nodes::AsmsNodes::default();
    for i in 1..=7 {
        // let output_name = format!("M2_segment_{i}_axial_d");
        let output_name = format!("MC_M2_S{i}_VC_delta_D");
        println!("Loading nodes from {output_name}");
        let idx = Box::<dyn fem_io::GetOut>::try_from(output_name.clone())
            .map(|x| x.position(&fem.outputs))?
            .expect(&format!(
                "failed to find the index of the output: {output_name}"
            ));
        let xyz = fem.outputs[idx]
            .as_ref()
            .map(|i| i.get_by(|i| i.properties.location.clone()))
            .expect(&format!(
                "failed to read nodes locations from {output_name}"
            ));
        nodes.push(nodes::Nodes { sid: i as u8, xyz });
    }

    nodes.into_bin("ASMS-nodes.bin")?;
    // nodes.into_parquet("ASMS-nodes.parquet")?;

    Ok(())
}
