use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    cc::Build::new()
        .cuda(true)
        .file("src/solver.cu")
        .compile("femcudasolver");
    bindgen::builder()
        .header("src/solver.hpp")
        .clang_arg("-I/usr/local/cuda/include")
        .allowlist_type("mode_state_space")
        .allowlist_type("state_space")
        .generate()?
        .write_to_file("src/bindings.rs")?;
    println!("cargo::rerun-if-changed=src/solver.cu");
    println!("cargo::rerun-if-changed=src/solver.h");
    Ok(())
}
