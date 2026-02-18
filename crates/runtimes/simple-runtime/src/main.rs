use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser as _;
use config::Config;
use html::HtmlRenderer;
use log::{error, info, warn};
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

use crate::{
    settings::{Renderer, Settings},
    setup::Models,
    update::{check_crate_version, check_cuda},
};

mod api;
mod cache;
pub mod cli;
mod debug;
mod dict;
mod execute;
pub mod settings;
pub mod setup;
mod ui;
mod update;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    let (level, ort_level) = match cli.verbose {
        3 | 2 => ("debug", "ort=debug"),
        1 => ("info", "ort=warn"),
        _ => ("warn", "ort=error"),
    };

    let base_filter = EnvFilter::new(level);
    let filter = match ort_level.parse() {
        Ok(directive) => base_filter.add_directive(directive),
        Err(err) => {
            eprintln!("Failed to parse ORT log directive '{ort_level}': {err}");
            return ExitCode::FAILURE;
        }
    };

    tracing_subscriber::fmt()
        .with_level(true)
        .with_target(true)
        .with_env_filter(filter)
        .init();
    let cuda = check_cuda();
    if !cuda && cfg!(all(target_arch = "x86_64", not(target_os = "macos"))) {
        warn!("CUDA is not available")
    }
    let _ = check_crate_version("frederik-uni/manga-image-translator-rust").await;

    let mut models = Models::new(
        cli.max_batch_size_upscaler,
        cli.max_batch_size_ocr,
        true,
        cuda,
    )
    .await;
    match cli.command {
        cli::Commands::Cli {
            input,
            output,
            config,
            overwrite,
        } => {
            let mut input_list = WalkDir::new(&input)
                .into_iter()
                .filter_map(|v| v.ok())
                .map(|v| v.path().to_path_buf())
                .filter(|v| v.is_file())
                .filter(|v| !v.to_string_lossy().starts_with("."))
                .map(|v| v.strip_prefix(&input).map(|v| v.to_path_buf()).unwrap_or(v))
                //TODO: add other extensions
                .filter(|v| {
                    ["png", "jpg", "jpeg", "webp"].contains(
                        &v.extension()
                            .map(|v| v.to_string_lossy())
                            .unwrap_or_default()
                            .to_lowercase()
                            .as_str(),
                    )
                })
                .collect::<Vec<_>>();
            let mut settings = Config::builder();
            if let Some(config) = config {
                if !config.exists() {
                    error!("Config file does not exist: {}", config.display());
                    return ExitCode::FAILURE;
                }
                settings = settings.add_source(config::File::from(config));
            }
            let settings = match settings.build() {
                Ok(settings) => settings,
                Err(err) => {
                    error!("Failed to build settings: {err}");
                    return ExitCode::FAILURE;
                }
            };
            let settings = settings.try_deserialize::<Settings>().unwrap_or_default();
            let out_ext = settings.render.renderer.extension();
            if !overwrite {
                input_list = input_list
                    .into_iter()
                    .filter(|v| {
                        let mut path = output.join(v);
                        path.set_extension(out_ext);
                        !path.exists()
                    })
                    .collect::<Vec<_>>();
            }

            for path in input_list {
                info!("Processing {}", path.display());
                let mut output = output.join(&path);
                let path = input.join(path);
                if !path.exists() || !path.is_file() {
                    warn!("File {} cant be found", path.display());
                    continue;
                }
                let img = match image::open(&path) {
                    Ok(img) => img,
                    Err(err) => {
                        error!("Failed to open image {}: {}", path.display(), err);
                        continue;
                    }
                };
                let debug_path = if cli.verbose > 2 {
                    let id = uuid::Uuid::new_v4();
                    let p = PathBuf::from(format!("debug/{}", id.to_string()));
                    if let Err(err) = create_dir_all(&p) {
                        error!("Failed to create debug directory {}: {}", p.display(), err);
                        continue;
                    }
                    Some(p)
                } else {
                    None
                };
                let exp = match models.execute(img, &settings, debug_path).await {
                    Ok(exp) => exp,
                    Err(err) => {
                        error!("Failed to process image {}: {}", path.display(), err);
                        continue;
                    }
                };
                let exp = match exp {
                    Some(v) => v,
                    None => {
                        info!("Failed to detect any translatable content");
                        continue;
                    }
                };
                output.set_extension(out_ext);
                if settings.render.renderer == Renderer::Html {
                    let (data, _) = HtmlRenderer::render(vec![exp], None, false);
                    if let Some(parent) = output.parent() {
                        if let Err(err) = create_dir_all(parent) {
                            error!(
                                "Failed to create parent directory {}: {}",
                                parent.display(),
                                err
                            );
                            continue;
                        }
                        if let Err(err) = html::copy_files(parent) {
                            error!(
                                "Failed to copy html assets to {}: {}",
                                parent.display(),
                                err
                            );
                            continue;
                        }
                    }
                    match File::create(&output).and_then(|mut file| file.write_all(&data)) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to write output file {}: {}", output.display(), err);
                            continue;
                        }
                    }
                } else {
                    let bin = exp.export();
                    if let Some(parent) = output.parent() {
                        if let Err(err) = create_dir_all(parent) {
                            error!(
                                "Failed to create parent directory {}: {}",
                                parent.display(),
                                err
                            );
                            continue;
                        }
                    }
                    match File::create(&output).and_then(|mut file| file.write_all(&bin)) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to write output file {}: {}", output.display(), err);
                            continue;
                        }
                    }
                }
            }
        }
        cli::Commands::Api { host, port } => {
            if let Err(err) = api::main(&host, port).await {
                error!("API server failed: {}", err);
                return ExitCode::FAILURE;
            }
        }
        cli::Commands::Ui => {
            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0]),
                // .with_icon(
                //     // NOTE: Adding an icon is optional
                //     eframe::icon_data::from_png_bytes(
                //         &include_bytes!("../assets/icon-256.png")[..],
                //     )
                //     .expect("Failed to load icon"),
                // ),
                ..Default::default()
            };
            if let Err(err) = eframe::run_native(
                "Manga Image Translator",
                native_options,
                Box::new(|cc| Ok(Box::new(ui::MitApp::new(cc)))),
            ) {
                error!("Failed to run egui: {}", err);
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
