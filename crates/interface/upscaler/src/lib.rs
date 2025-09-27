use std::sync::Arc;

use interface_image::{ImageOp, RawImage};

#[async_trait::async_trait]
pub trait Upscaler {
    async fn upscale(
        &self,
        image: &RawImage,
        patch_size: Option<usize>,
        padding: usize,
        img_processor: &Arc<dyn ImageOp + Send + Sync>,
    ) -> anyhow::Result<RawImage>;
}
