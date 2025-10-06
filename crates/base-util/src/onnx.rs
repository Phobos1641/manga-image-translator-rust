use log::{info, Level};
use ndarray::{Array, Array2, IxDyn};
use ort::{
    execution_providers::{
        ArenaExtendStrategy, CUDAExecutionProvider, CoreMLExecutionProvider,
        DirectMLExecutionProvider, ROCmExecutionProvider, TensorRTExecutionProvider,
    },
    session::{
        builder::{GraphOptimizationLevel, SessionBuilder},
        Session,
    },
};

#[derive(Clone, Debug)]
pub enum Providers {
    TensorRT,
    CUDA,
    DirectML,
    CoreML,
    RocM,
}

pub fn all_providers() -> Vec<Providers> {
    vec![
        Providers::CUDA,
        Providers::RocM,
        Providers::TensorRT,
        #[cfg(target_os = "windows")]
        Providers::DirectML,
        #[cfg(target_os = "macos")]
        Providers::CoreML,
    ]
}

pub fn gpu_providers() -> Vec<Providers> {
    vec![
        Providers::CUDA,
        Providers::RocM,
        Providers::TensorRT,
        #[cfg(target_os = "windows")]
        Providers::DirectML,
        Providers::RocM,
    ]
}

pub fn new_session(providers: &[Providers]) -> anyhow::Result<SessionBuilder> {
    Ok(new_session_(Session::builder()?, providers)?)
}

pub fn new_session_(
    session_builder: SessionBuilder,
    providers: &[Providers],
) -> Result<SessionBuilder, ort::Error> {
    let session_builder = session_builder
        .with_logger(Box::new(
            |level: ort::logging::LogLevel,
             category: &str,
             id: &str,
             code_location: &str,
             message: &str| {
                let log_level = match level {
                    ort::logging::LogLevel::Verbose => Level::Trace,
                    ort::logging::LogLevel::Info => Level::Info,
                    ort::logging::LogLevel::Warning => Level::Warn,
                    ort::logging::LogLevel::Error => Level::Error,
                    ort::logging::LogLevel::Fatal => Level::Error,
                };

                log::log!(
                    log_level,
                    "[ORT][{category}][{id}] {message} (at {code_location})"
                );
            },
        ))?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_parallel_execution(true)?
        .with_intra_threads(4)?
        .with_inter_threads(2)?;
    for provider_ in providers {
        let provider = match provider_ {
            Providers::TensorRT => TensorRTExecutionProvider::default()
                .with_device_id(0)
                .build(),
            Providers::CUDA => CUDAExecutionProvider::default()
                .with_device_id(0)
                .with_arena_extend_strategy(ArenaExtendStrategy::SameAsRequested)
                .build(),
            Providers::DirectML => DirectMLExecutionProvider::default()
                .with_device_id(0)
                .build(),
            Providers::CoreML => CoreMLExecutionProvider::default()
                .with_model_cache_dir("models/cache")
                .with_compute_units(ort::execution_providers::coreml::CoreMLComputeUnits::All)
                .build(),
            Providers::RocM => ROCmExecutionProvider::default().with_device_id(0).build(),
        }
        .error_on_failure();
        if let Ok(session_builder) = session_builder
            .clone()
            .with_execution_providers(vec![provider])
        {
            info!("Execution provider {:?} in use", provider_);
            return Ok(session_builder);
        }
    }
    info!("Execution provider CPU in use");
    Ok(session_builder)
}

pub fn dyn_to_2d(arr: Array<f32, IxDyn>) -> Option<Array2<f32>> {
    if arr.ndim() == 2 {
        let shape = arr.shape();
        let (rows, cols) = (shape[0], shape[1]);

        arr.into_shape((rows, cols)).ok()
    } else {
        None
    }
}
