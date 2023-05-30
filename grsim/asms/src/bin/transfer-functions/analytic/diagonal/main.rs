use asms::Sys;
use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

fn main() -> anyhow::Result<()> {
    /*     env::set_var(
        "DATA_REPO",
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("bin")
            .join("transfer-functions")
            .join("analytic"),
    ); */
    let repo = env::var("DATA_REPO").unwrap();
    let path = Path::new(&repo).join("sys0001.bin");

    let file = File::open(&path)?;
    let mut buffer = BufReader::new(file);
    let sys: Sys = bincode::deserialize_from(&mut buffer)?;

    dbg!(sys.frequencies());
    dbg!(sys.get((0, 0)));

    /*     let mut tfs = vec![];
    let mut nu = Option::<Vec<f64>>::None;
    for i in 0..6 {
        let (nu_tf, tf) = sys.get_map((i, i), |x| x.norm()).unwrap();
        if let None = nu {
            nu = Some(nu_tf);
        }
        tfs.push(tf);
    }
    let data = (nu.unwrap(), tfs);
    let path = Path::new(&repo).join("asm_tf1.pkl");
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    serde_pickle::to_writer(&mut buffer, &data, Default::default())?; */

    Ok(())
}
