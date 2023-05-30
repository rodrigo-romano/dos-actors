use asms::{FrequencyResponse, ASM};
use std::{env, fs::File, io::BufWriter, path::Path, time::Instant};

fn main() -> anyhow::Result<()> {
    let nu = env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .expect("AWS_BATCH_JOB_ARRAY_INDEX env var missing")
        .parse::<usize>()
        .expect("AWS_BATCH_JOB_ARRAY_INDEX parsing failed");

    let sid = 1;
    let asm = ASM::new(sid)?;

    println!("Evaluating ASM transfer function ...");
    let now = Instant::now();
    let sys = asm.frequency_response(nu as f64);
    println!(" completed in {}s", now.elapsed().as_secs());

    let repo = env::var("DATA_REPO").expect("DATA_REPO not set");
    let path = Path::new(&repo).join(format!("sys{:04}.bin", nu));
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    bincode::serialize_into(&mut buffer, &sys)?;

    Ok(())
}
