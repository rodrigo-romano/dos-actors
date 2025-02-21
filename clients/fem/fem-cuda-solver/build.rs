use std::{env, error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    cc::Build::new()
        .cuda(true)
        .file("src/solver.cu")
        .compile("femcudasolver");
    println!("cargo:rustc-link-lib=cublas");
    bindgen::builder()
        .header("src/solver.hpp")
        .clang_arg("-I/usr/local/cuda/include")
        .allowlist_type("mode_state_space")
        .allowlist_type("state_space")
        .generate()?
        .write_to_file(Path::new(&env::var("OUT_DIR")?).join("bindings.rs"))?;
    println!("cargo::rerun-if-changed=src/solver.cu");
    println!("cargo::rerun-if-changed=src/solver.hpp");
    Ok(())
}
