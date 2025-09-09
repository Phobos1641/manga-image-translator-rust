use std::sync::Arc;

use interface_image::{ImageOp, RawImage};

pub trait Upscaler {
    fn upscale(
        &mut self,
        image: &RawImage,
        patch_size: Option<usize>,
        padding: usize,
        img_processor: &Arc<dyn ImageOp + Send + Sync>,
    ) -> anyhow::Result<RawImage>;
}
