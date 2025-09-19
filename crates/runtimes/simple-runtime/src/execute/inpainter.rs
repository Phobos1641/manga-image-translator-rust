use std::sync::Arc;

use interface_image::{Mask, RawImage};
use interface_inpainter::InpainterOptions;

use crate::{execute::ImageProcessor, settings::InpainterSettings, setup::Models};

impl Models {
    pub fn run_inpainter(
        &mut self,
        img: &Arc<RawImage>,
        mask: Mask,
        config: &InpainterSettings,
        ip: &ImageProcessor,
    ) -> anyhow::Result<RawImage> {
        let inpainted = self.get_inpainter(config.inpainter).inpaint(
            img,
            mask,
            InpainterOptions {
                inpainting_size: config.inpainting_size,
                color: config.inpaint_color,
            },
            &ip,
        )?;
        Ok(inpainted)
    }
}
