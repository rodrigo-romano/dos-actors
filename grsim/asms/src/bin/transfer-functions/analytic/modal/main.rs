use asms::{if64, Sys};
use matio_rs::MatFile;
use num_complex::Complex;
use std::{env, fs::File, io::BufWriter, path::Path};

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
    let path = Path::new(&repo).join("sys.tgz");

    let sys = Sys::from_tarball(&path)?;

    let sid = 1u8;
    let kl_modes: nalgebra::DMatrix<f64> =
        MatFile::load("KLmodesGS36.mat")?.var(format!("KL_{sid}"))?;
    let kl_modes: nalgebra::DMatrix<if64> = kl_modes.map(|x| Complex::new(x, 0f64));

    let modal_sys = kl_modes.transpose() * &sys * &kl_modes;
    let modal_frs = modal_sys.diagonals();
    let nu = modal_sys.frequencies();

    let path = Path::new(&repo).join("asm_kl_fr.pkl");
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    serde_pickle::to_writer(&mut buffer, &(nu, modal_frs), Default::default())?;

    Ok(())
}
