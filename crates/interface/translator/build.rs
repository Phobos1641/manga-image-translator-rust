use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut target_dir = out_dir.clone();
    for _ in 0..4 {
        target_dir = target_dir.parent().unwrap().to_path_buf();
    }

    let cudnn_file = ("libcudnn.so", "libcudnn.so.9");

    #[cfg(target_os = "windows")]
    let cudnn_file = ("cudnn64_9.dll", "cudnn64_9.dll");

    let vendor_lib = target_dir
        .join("ctranslate2-vendor")
        .join("dyn")
        .join(cudnn_file.0);

    if vendor_lib.exists() {
        let dest = target_dir.join(cudnn_file.1);

        if let Err(e) = fs::copy(&vendor_lib, &dest) {
            println!("cargo:warning=Failed to copy {}: {e}", cudnn_file.0);
        } else {
            println!(
                "cargo:warning=Copied {} from {} to {}",
                cudnn_file.0,
                vendor_lib.display(),
                dest.display()
            );
        }

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    } else {
        println!(
            "cargo:warning=No {} found at {}, skipping copy",
            cudnn_file.0,
            vendor_lib.display()
        );
    }
}
