use std::{fmt::Display, sync::Arc};

use base_util::onnx::{new_session, Providers};
use half::f16;
use interface_image::RawImage;
use interface_model::{
    impl_model_helpers, impl_model_load_helpers, Model, ModelLoad, ModelRead, ModelWrap,
};
use interface_upscaler::Upscaler;
use maplit::hashmap;
use ndarray::{ArrayView3, ArrayViewD, Axis};
use ort::{inputs, session::RunOptions, value::Tensor};
use ort_parallel::AsyncSessionPool;
use util::spawn_blocking;

pub struct Anime4KUpscaler {
    model: ModelWrap<AsyncSessionPool>,
    model_kind: Anime4KModel,
    providers: Arc<Vec<Providers>>,
}

impl Anime4KUpscaler {
    pub fn new(model_kind: Anime4KModel, providers: Arc<Vec<Providers>>) -> Self {
        Anime4KUpscaler {
            model: Default::default(),
            model_kind,
            providers,
        }
    }
}

pub enum Anime4KModel {
    X4UUL,
    X4UL,
    X3VL,
    X3L,
    X2M,
    X2S,
}

impl Display for Anime4KModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Anime4KModel::X4UUL => write!(f, "2x_UUL"),
            Anime4KModel::X4UL => write!(f, "4x_UL"),
            Anime4KModel::X3VL => write!(f, "3x_VL"),
            Anime4KModel::X3L => write!(f, "3x_L"),
            Anime4KModel::X2M => write!(f, "2x_M"),
            Anime4KModel::X2S => write!(f, "2x_S"),
        }
    }
}

#[async_trait::async_trait]
impl ModelLoad for Anime4KUpscaler {
    impl_model_load_helpers!(model, AsyncSessionPool);

    async fn reload(&self) -> anyhow::Result<ModelRead<'_, Self::T>> {
        let model = self.model_kind.to_string();
        let path = self
            .download_model(&model, &format!("{model}.onnx"))
            .await?;
        let session = AsyncSessionPool::commit_from_file(new_session(&self.providers)?, &path, 10)?;
        *self.model.write().await = Some(session);
        Ok(self.get_model().await.expect("loaded before"))
    }
}

impl Model for Anime4KUpscaler {
    impl_model_helpers!("upscaler", "waifu2x", model);

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        hashmap! {
            "2x_S" => interface_model::ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/anime4k/2x_S", hash: "###" },
            "2x_M" => interface_model::ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/anime4k/2x_M", hash: "###" },
            "3x_L" => interface_model::ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/anime4k/3x_L", hash: "###" },
            "3x_VL" => interface_model::ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/anime4k/3x_VL", hash: "###" },
            "4x_UL" => interface_model::ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/anime4k/4x_UL", hash: "###" },
            "4x_UUL" =>interface_model::ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/anime4k/4x_UUL", hash: "###" }
        }
    }
}

#[async_trait::async_trait]
impl Upscaler for Anime4KUpscaler {
    async fn upscale(
        &self,
        image: &RawImage,
        _: Option<usize>,
        _: usize,
        _: &Arc<dyn interface_image::ImageOp + Send + Sync>,
    ) -> anyhow::Result<RawImage> {
        let t = spawn_blocking!(|| {
            let image = image
                .as_ndarray()?
                .mapv(|v| f16::from_f32(v as f32 / 255.0))
                .permuted_axes((2, 0, 1))
                .insert_axis(Axis(0));
            let t = Tensor::from_array(image)?;
            Ok::<_, anyhow::Error>(t)
        })??;

        let model = self.load().await?;
        let settings = RunOptions::new()?;
        let out = model.run_async(inputs! {"input"=>t}, &settings).await?;
        let out = spawn_blocking!(|| {
            let out: ArrayViewD<f16> = out[0].try_extract_array()?.remove_axis(Axis(0));
            let out: ArrayView3<f16> = out.into_dimensionality()?;
            let out = out
                .mapv(|v| (v.to_f32() * 255.0) as u8)
                .permuted_axes((1, 2, 0));
            Ok::<_, anyhow::Error>(RawImage::from(out))
        })??;

        Ok(out)
    }
}
