use std::{collections::HashMap, sync::Arc};

use interface_inpainter::{colorize_mask_area, Inpainter, InpainterOptions};
use interface_model::Model;
use tokio::sync::Mutex;
use util::spawn_blocking;

pub struct ColorInpainter {
    loaded: Arc<Mutex<bool>>,
}

impl Default for ColorInpainter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorInpainter {
    pub fn new() -> Self {
        Self {
            loaded: Arc::new(Mutex::new(true)),
        }
    }
}

#[async_trait::async_trait]
impl Model for ColorInpainter {
    fn name(&self) -> &'static str {
        "color"
    }

    fn kind(&self) -> &'static str {
        "inpainter"
    }

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        HashMap::new()
    }

    async fn unload(&self) {
        *self.loaded.lock().await = false;
    }

    async fn loaded_(&self) -> bool {
        *self.loaded.lock().await
    }

    async fn reload_(&self) -> anyhow::Result<()> {
        *self.loaded.lock().await = true;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Inpainter for ColorInpainter {
    async fn inpaint(
        &self,
        image: &interface_image::RawImage,
        mask: interface_image::Mask,
        options: InpainterOptions,
        _: &Arc<dyn interface_image::ImageOp + Send + Sync>,
    ) -> anyhow::Result<interface_image::RawImage> {
        Ok(spawn_blocking!(|| colorize_mask_area(
            // allow:clone[change inplace]
            image.clone(),
            &mask,
            options.color,
        ))?)
    }
}

#[cfg(test)]
mod tests {
    use interface_image::{CpuImageProcessor, ImageOp, Mask, RawImage};
    use ndarray::Array2;

    use super::*;

    #[tokio::test]
    async fn test_inpaint() {
        let img = RawImage::new("./imgs/232265329-6a560438-e887-4f7f-b6a1-a61b8648f781.png")
            .expect("Failed to load image");
        let img = RawImage::from(img);
        let img_processor =
            Arc::new(CpuImageProcessor::default()) as Arc<dyn ImageOp + Send + Sync>;
        let mask: Array2<u8> = ndarray_npy::read_npy("../lama_large/mask.npy").unwrap();
        let mask = Mask::from(mask);
        let inp = ColorInpainter::default();
        let v = inp
            .inpaint(&Arc::new(img), mask, Default::default(), &img_processor)
            .await
            .unwrap();
        v.to_image().unwrap().save("inpainted.png").unwrap()
    }
}
