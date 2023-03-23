use gmt_fem::FEM;
use polars::prelude::*;

fn main() -> anyhow::Result<()> {
    let fem = FEM::from_env()?;
    println!("{fem}");

    let mut series = vec![];
    for i in 1..=7 {
        println!("Loading nodes from M2Segment{i}AxialD");
        let nodes = fem.outputs[9 + i]
            .as_ref()
            .map(|i| i.get_by(|i| i.properties.location.clone()))
            .unwrap();

        let s: Vec<_> = nodes
            .iter()
            .map(|xyz| {
                let s: Series = xyz.iter().collect();
                s
            })
            .collect();
        series.push(Series::new(&format!("S{i}"), s));
    }

    let mut df = DataFrame::new(series)?;
    println!("{}", df.head(None));

    let mut file = std::fs::File::create("ASMS-nodes.parquet")?;
    ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok(())
}
