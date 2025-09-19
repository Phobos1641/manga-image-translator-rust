use interface_detector::textlines::Quadrilateral;
use interface_image::{Mask, RawImage};

use crate::{execute::ImageProcessor, settings::DetectorSettings, setup::Models};

impl Models {
    pub fn run_detector(
        &mut self,
        img: &RawImage,
        config: &DetectorSettings,
        ip: &ImageProcessor,
    ) -> anyhow::Result<(Vec<Quadrilateral>, Mask)> {
        let (areas, mask) = self.get_detector(config.detector).detect(
            img,
            config.preprocessor,
            config.options,
            ip,
        )?;
        Ok((areas, mask))
    }
}
