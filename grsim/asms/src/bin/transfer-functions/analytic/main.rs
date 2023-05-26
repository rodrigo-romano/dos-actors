use asms::{Frequencies, FrequencyResponse, ASM};
use matio_rs::MatFile;
use std::{env, fs::File, path::Path, time::Instant};

fn main() -> anyhow::Result<()> {
    env::set_var(
        "DATA_REPO",
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("bin")
            .join("transfer-functions")
            .join("analytic"),
    );

    let sid = 1;

    let kl_modes: nalgebra::DMatrix<f64> =
        MatFile::load("KLmodesGS36.mat")?.var(format!("KL_{sid}"))?;
    dbg!(kl_modes.shape());

    let asm = ASM::new(sid)?.modes(kl_modes);

    println!("Evaluating ASM transfer function ...");
    let now = Instant::now();
    let (nu, asm_tf) = asm.frequency_response(Frequencies::LinSpace {
        lower: 1e1,
        upper: 4e3,
        n: 4000 - 10 + 1,
    });
    println!(" completed in {}s", now.elapsed().as_secs());

    dbg!(asm_tf[0].shape());

    let mut file = File::create("asm_tf.pkl")?;
    serde_pickle::to_writer(&mut file, &(nu, asm_tf), Default::default())?;

    Ok(())
}
