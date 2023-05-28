use asms::{FrequencyResponse, ASM};
use matio_rs::MatFile;
use std::{env, fs::File, io::BufWriter, path::Path, time::Instant};

fn main() -> anyhow::Result<()> {
    env::set_var(
        "DATA_REPO",
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("bin")
            .join("transfer-functions")
            .join("analytic"),
    );
    let repo = env::var("DATA_REPO").unwrap();

    let sid = 1;

    let kl_modes: nalgebra::DMatrix<f64> =
        MatFile::load("KLmodesGS36.mat")?.var(format!("KL_{sid}"))?;
    dbg!(kl_modes.shape());

    let asm = ASM::new(sid)?.modes(kl_modes);

    let nu_first = 10f64; // Hz
    let nu_last = 4_000_f64; // Hz
    let s: usize = env::args()
        .skip(1)
        .next()
        .and_then(|s| s.parse().ok())
        .expect("expected 1 integer argument in the range [0,3], found none");
    let nu: Vec<_> = (0..4000)
        .skip(dbg!(s))
        .step_by(4)
        .map(|i| nu_first + i as f64)
        .take_while(|&x| x <= nu_last)
        .collect();
    dbg!(nu.len());

    println!("Evaluating ASM transfer function ...");
    let now = Instant::now();
    let sys = asm.frequency_response(nu);
    println!(" completed in {}s", now.elapsed().as_secs());

    let path = Path::new(&repo).join(format!("sys#{s}.pkl)"));
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    bincode::serialize_into(&mut buffer, &sys)?;
    // serde_pickle::to_writer(&mut file, &(nu, asm_tf), Default::default())?;

    Ok(())
}
