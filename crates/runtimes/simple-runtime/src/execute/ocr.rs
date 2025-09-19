use std::sync::Arc;

use interface_detector::textlines::Quadrilateral;
use interface_image::RawImage;
use interface_ocr::QuadrilateralInfo;
use parking_lot::Mutex;

use crate::{execute::ImageProcessor, settings::OCRSettings, setup::Models};

impl Models {
    pub async fn run_ocr(
        &mut self,
        img: &Arc<RawImage>,
        areas: &[Arc<Mutex<Quadrilateral>>],
        config: &OCRSettings,
        ip: &ImageProcessor,
    ) -> anyhow::Result<Vec<QuadrilateralInfo>> {
        let textlines = self.get_ocr(config.ocr).detect(img, areas, ip).await?;
        Ok(textlines)
    }
}
