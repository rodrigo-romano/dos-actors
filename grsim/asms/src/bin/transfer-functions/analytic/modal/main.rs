use asms::{if64, Sys};
use matio_rs::MatFile;
use num_complex::Complex;
use std::{env, fs::File, io::BufWriter, path::Path, time::Instant};

fn main() -> anyhow::Result<()> {
    let repo = env::var("DATA_REPO").unwrap();
    let path = Path::new(&repo).join("asm-structural-dc-mismatch_delay:2_sys.tgz");

    println!("Loading system from {:?}", path);
    let now = Instant::now();
    let sys = Sys::from_tarball(&path)?;
    println!("System loaded in {}s", now.elapsed().as_secs());

    let sid = 1u8;
    let kl_modes: nalgebra::DMatrix<f64> =
        MatFile::load("KLmodesGS36.mat")?.var(format!("KL_{sid}"))?;
    let kl_modes: nalgebra::DMatrix<if64> = kl_modes.map(|x| Complex::new(x, 0f64));

    println!("Projecting the system into the Karhunen-Loeve basis");
    let now = Instant::now();
    let modal_sys = kl_modes.transpose() * &sys * &kl_modes;
    println!(
        "System transformation performed in {}s",
        now.elapsed().as_secs()
    );

    println!("Extracting MIMOs diagonals");
    let now = Instant::now();
    let modal_frs = modal_sys.diagonals();
    println!("MIMOs diagonals extracted in {}s", now.elapsed().as_secs());
    let nu = modal_sys.frequencies();

    let path = Path::new(&repo).join("asm-structural-dc-mismatch_delay:2_kl_fr.pkl");
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    serde_pickle::to_writer(&mut buffer, &(nu, modal_frs), Default::default())?;

    Ok(())
}
