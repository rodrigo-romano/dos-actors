use std::env;

fn main() {
    if option_env!("FEM_REPO").is_some() {
        println!("cargo:rustc-cfg=fem");
        let (input_names, _output_names) = gmt_fem_code_builder::io_names(env!("CARGO_PKG_NAME"))
            .expect("failed to get FEM inputs/ouputs names");
        if input_names.find("OSSGIRTooth6F").is_some() {
            println!("cargo:warning=OSSGIRTooth6F input detected in gmt_dos-client_mount crate");
            println!("cargo:rustc-cfg=gir_tooth")
        }
    }

    let mount_model = match env::var("MOUNT_MODEL") {
        Ok(val) => val,
        Err(_) => {
            println!("cargo:warning=setting `MOUNT_MODEL=MOUNT_FDR_1kHz`");
            env::set_var("MOUNT_MODEL", "MOUNT_FDR_1kHz");
            "MOUNT_FDR_1kHz".to_string()
        }
    };

    if mount_model.contains("FDR") {
        println!(r#"cargo:rustc-cfg=mount="FDR""#);
        if mount_model == "MOUNT_FDR_1kHz-az17Hz" {
            println!("cargo:warning=compiling ODC mount control with 17Hz notch filter");
        } else {
            println!("cargo:warning=compiling vanilla ODC mount FDR model");
        }
    }
    if mount_model.contains("PDR") {
        println!(r#"cargo:rustc-cfg=mount="PDR""#);
        println!("cargo:warning=compiling vanilla ODC mount PDR model");
    }
    println!("cargo:rerun-if-env-changed=MOUNT_MODEL");
    println!("cargo:rerun-if-env-changed=FEM_REPO");
}
