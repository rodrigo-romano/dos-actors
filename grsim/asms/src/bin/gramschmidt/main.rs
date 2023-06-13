use std::{env::args, fs::File, path::Path, time::Instant};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = args().collect();
    let file_name = args.get(1).expect("filename missing");
    let n_mode: usize = args.get(2).expect("# of modes missing").parse()?;

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin")
        .join("gramschmidt")
        .join(file_name);
    println!("Loading from {:?}", path);
    let file = File::open(path)?;
    let data: Vec<f64> = serde_pickle::from_reader(&file, Default::default())?;

    print!("Orthonormalizing");
    let now = Instant::now();
    let gsed_data = zernike::gram_schmidt(&data, n_mode);
    println!(" in {}ms", now.elapsed().as_millis());

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin")
        .join("gramschmidt")
        .join(file_name)
        .with_extension("")
        .with_extension("gs.pkl");
    println!("Saving to {:?}", path);
    let mut file = File::create(path)?;
    serde_pickle::to_writer(&mut file, &gsed_data, Default::default())?;

    Ok(())
}
