use asms::{BuilderTrait, FrequencyResponse, Sys, ASM};
use std::{env, fs::File, io::BufWriter, path::Path, time::Instant};

/*
Environment variables:
 * AWS_BATCH_JOB_ARRAY_INDEX: frequency [Hz] <usize>
 * ENABLE_DC_MISMATCH_COMPENSATION: anything or not present
 * DC_MISMATCH_DELAY: set the delay in secondss as 1/(DC_MISMATCH_DELAY x 8000) <usize>
 */

fn main() -> anyhow::Result<()> {
    let nu = env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .expect("AWS_BATCH_JOB_ARRAY_INDEX env var missing")
        .parse::<usize>()
        .expect("AWS_BATCH_JOB_ARRAY_INDEX parsing failed");

    let sid = 1;
    let sim_sampling_frequency = 8e3_f64;
    let mut file_name = String::from("asm-structural");

    let asm = if let Ok(_) = env::var("ENABLE_DC_MISMATCH_COMPENSATION") {
        let value = env::var("DC_MISMATCH_DELAY")
            .ok()
            .and_then(|value| value.parse::<usize>().ok());
        let maybe_delay = value.map(|value| sim_sampling_frequency.recip() / value as f64);
        if let Some(delay) = maybe_delay {
            println!(
                "ASM model with {:.4}ms delay DC mismatch compensation scheme",
                delay * 1e3
            );
            file_name = format!("asm-structural-dc-mismatch_delay:{}", value.unwrap());
        } else {
            println!("ASM model with no delay DC mismatch compensation scheme");
            file_name = format!("asm-structural-dc-mismatch_no-delay");
        };
        ASM::builder(sid)
            .enable_static_gain_mismatch_compensation(maybe_delay)
            .filename(&file_name)
    } else {
        println!("ASM model without DC mismatch compensation scheme");
        ASM::builder(sid).filename(&file_name)
    }
    .build()?;

    println!("Evaluating ASM transfer function ...");
    let now = Instant::now();
    let sys: Sys = asm.frequency_response(nu as f64).into();
    println!(" completed in {}s", now.elapsed().as_secs());

    let repo = env::var("DATA_REPO").expect("DATA_REPO not set");
    let path = Path::new(&repo).join(format!("{}_sys{:04}.bin", file_name, nu));
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    bincode::serialize_into(&mut buffer, &sys)?;

    Ok(())
}
