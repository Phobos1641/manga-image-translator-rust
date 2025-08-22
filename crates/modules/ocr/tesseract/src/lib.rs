use std::collections::HashMap;

use image::{DynamicImage, GenericImageView as _, RgbImage};
use interface_image::Mask;
use interface_model::{impl_model_load_helpers, Model, ModelLoad};
use interface_ocr::{Ocr, QuadrilateralInfo};
use uni_ocr::{Language, OcrEngine, OcrOptions, OcrProvider};

#[derive(Default)]
pub struct TesseractOCR {
    model: Option<OcrEngine>,
}

impl TesseractOCR {}

impl ModelLoad for TesseractOCR {
    type T = OcrEngine;

    fn loaded(&self) -> bool {
        self.model.is_some()
    }

    fn get_model(&mut self) -> Option<&mut Self::T> {
        self.model.as_mut()
    }

    fn reload(&mut self) -> Result<&mut Self::T, interface_model::ModelLoadError> {
        let engine = OcrEngine::new(OcrProvider::Tesseract)
            .unwrap()
            .with_options(OcrOptions::default().languages(vec![
                Language::Chinese,
                Language::Japanese,
                Language::Korean,
                Language::English,
            ]));
        self.model = Some(engine);
        Ok(self.model.as_mut().unwrap())
    }
}

impl Model for TesseractOCR {
    impl_model_load_helpers!("ocr", "tesseract");

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        HashMap::new()
    }

    fn unload(&mut self) {
        self.model = None;
    }
}

#[async_trait::async_trait]
impl Ocr for TesseractOCR {
    async fn detect(
        &mut self,
        image: &interface_image::RawImage,
        areas: &[interface_detector::textlines::Quadrilateral],
        img_processor: &Box<dyn interface_image::ImageOp + Send + Sync>,
    ) -> anyhow::Result<Vec<interface_ocr::QuadrilateralInfo>> {
        let mut texts = vec![];
        let grayscale = DynamicImage::from(
            RgbImage::from_raw(image.width as u32, image.height as u32, image.data.clone())
                .unwrap(),
        )
        .to_luma8();
        for area in areas {
            let bbox = area.aabb();
            let view = grayscale.view(bbox.x as u32, bbox.y as u32, bbox.w as u32, bbox.h as u32);
            let img = Mask::from(view.to_image());
            texts.push(self.detect_patch(img, area.clone(), img_processor).await?);
        }

        Ok(texts)
    }

    async fn detect_patch(
        &mut self,
        sliced_image: interface_image::Mask,
        area: interface_detector::textlines::Quadrilateral,
        _: &Box<dyn interface_image::ImageOp + Send + Sync>,
    ) -> anyhow::Result<interface_ocr::QuadrilateralInfo> {
        let model = self.load()?;
        let image = image::DynamicImage::from(sliced_image.to_image().unwrap());

        let (result, _, _) = model.recognize_image(&image).await?;
        Ok(QuadrilateralInfo {
            text: result,
            fg: None,
            bg: None,
            pos: area,
        })
    }
}

#[cfg(test)]
mod tests {
    use interface_detector::textlines::Quadrilateral;
    use interface_image::{CpuImageProcessor, ImageOp, RawImage};
    use interface_ocr::Ocr as _;

    use crate::TesseractOCR;

    #[tokio::test]
    async fn ocr_test() {
        let img = RawImage::new("./imgs/232265329-6a560438-e887-4f7f-b6a1-a61b8648f781.png")
            .expect("Failed to load image");
        let mut mocr = TesseractOCR::default();
        let inp = vec![
            Quadrilateral::new(vec![(208, 4), (246, 4), (246, 192), (208, 192)], 1.0),
            Quadrilateral::new(vec![(76, 1788), (128, 1788), (128, 1930), (76, 1930)], 1.0),
        ];
        let ip = Box::new(CpuImageProcessor::default()) as Box<dyn ImageOp + Send + Sync>;
        let v = mocr.detect(&img, &inp, &ip).await.unwrap();
        assert_eq!(v[0].text, "そうだなあ・・・");
        assert_eq!(v.len(), 2);
    }
}
