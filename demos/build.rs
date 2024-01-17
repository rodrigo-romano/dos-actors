fn main() {
    println!(
        "cargo:rustc-env=FEM_REPO={:}",
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111")
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-cfg=fem");
    println!("cargo:rerun-if-env-changed=FEM_REPO");
}
