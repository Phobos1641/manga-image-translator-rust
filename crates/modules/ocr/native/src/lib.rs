use std::collections::HashMap;

use image::{DynamicImage, GenericImageView as _, GrayImage, RgbImage};
use interface_image::Mask;
use interface_model::{impl_model_load_helpers, Model, ModelLoad};
use interface_ocr::{Ocr, QuadrilateralInfo};
use uni_ocr::{OcrEngine, OcrProvider};

pub struct NativeOCR {
    model: Option<OcrEngine>,
}

impl NativeOCR {}

impl ModelLoad for NativeOCR {
    type T = OcrEngine;

    fn loaded(&self) -> bool {
        self.model.is_some()
    }

    fn get_model(&mut self) -> Option<&mut Self::T> {
        self.model.as_mut()
    }

    fn reload(&mut self) -> Result<&mut Self::T, interface_model::ModelLoadError> {
        let engine = OcrEngine::new(OcrProvider::Auto).unwrap();
        self.model = Some(engine);
        Ok(self.model.as_mut().unwrap())
    }
}

impl Model for NativeOCR {
    impl_model_load_helpers!("ocr", "native");

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        HashMap::new()
    }

    fn unload(&mut self) {
        self.model = None;
    }
}

impl Ocr for NativeOCR {
    fn detect(
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
            texts.push(self.detect_patch(img, area.clone(), img_processor)?);
        }

        Ok(texts)
    }

    fn detect_patch(
        &mut self,
        sliced_image: interface_image::Mask,
        area: interface_detector::textlines::Quadrilateral,
        _: &Box<dyn interface_image::ImageOp + Send + Sync>,
    ) -> anyhow::Result<interface_ocr::QuadrilateralInfo> {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(self.detect_batch_async(sliced_image, area))
    }
}

impl NativeOCR {
    pub async fn detect_batch_async(
        &mut self,
        sliced_image: interface_image::Mask,
        area: interface_detector::textlines::Quadrilateral,
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
