mod detector;
mod dict;
mod inpainter;
mod mask_refinement;
mod ocr;
mod textline_merge;
mod translator;
mod upscaler;

use std::{path::PathBuf, sync::Arc};

use export::Export;
use image::DynamicImage;
use interface_image::{CpuImageProcessor, ImageOp, RawImage};

use crate::{
    debug::{bbox::render_bboxes, save_img, save_json, save_mask},
    settings::Settings,
    setup::Models,
};

pub type ImageProcessor = Arc<dyn ImageOp + Sync + Send>;

impl Models {
    pub async fn execute(
        &mut self,
        img: DynamicImage,
        config: &Settings,
        debug_path: Option<PathBuf>,
    ) -> anyhow::Result<Export> {
        let ip = Arc::new(CpuImageProcessor::default()) as ImageProcessor;
        let orig_img = img.clone();
        let (img, alpha) = RawImage::rgba(img);
        let (img, alpha) = self.run_upscaler(img, alpha, config.upscaler, &ip)?;

        if let Some(debug_path) = &debug_path {
            save_json(config, &debug_path.join("0_config.json"))?;
            save_img(&img, &debug_path.join("0_input.png"))?;
        }

        let (areas, mask) = self.run_detector(&img, &config.detector, &ip)?;
        if let Some(debug_path) = &debug_path {
            save_mask(&mask, &debug_path.join("1_mask_raw.png"))?;
            save_json(&areas, &debug_path.join("1_quadrilateral.json"))?;
            render_bboxes(&img, &areas, debug_path)?;
        }

        let areas = areas.into_iter().map(to_mutex).collect::<Vec<_>>();
        let img = Arc::new(img);

        let textlines = self.run_ocr(&img, &areas, &config.ocr, &ip).await?;

        if let Some(debug_path) = &debug_path {
            save_json(&textlines, &debug_path.join("2_quadrilateral.json"))?;
        }

        let textblocks = self.run_textline_merge(
            &textlines,
            img.width,
            img.height,
            &config.ocr,
            &config.translator,
        )?;

        if let Some(debug_path) = &debug_path {
            save_json(&textblocks, &debug_path.join("3_textblock.json"))?;
        }

        let textblocks = self.run_pre_dict(textblocks, &config.translator)?;
        if let Some(debug_path) = &debug_path {
            if config.translator.pre_dict.is_some() {
                save_json(
                    &textlines,
                    &debug_path.join("3_textblock_predict_applied.json"),
                )?;
            }
        }

        let textblocks = self.run_translators(textblocks, &config.translator).await?;

        if let Some(debug_path) = &debug_path {
            save_json(
                &textblocks,
                &debug_path.join("4_textblocks_translated.json"),
            )?;
        }

        let mask_refined =
            Models::run_mask_refinement(&img, &mask, &textblocks, &config.mask_refinement, &ip)?;

        if let Some(debug_path) = &debug_path {
            save_mask(&mask_refined, &debug_path.join("4_mask_refined.png"))?;
        }

        let inpainted = self.run_inpainter(&img, mask_refined, &config.inpainter, &ip)?;

        if let Some(debug_path) = &debug_path {
            save_img(&inpainted, &debug_path.join("5_inpainted.png"))?;
        }

        Ok(Export::new(
            orig_img,
            match alpha {
                Some(a) => inpainted.add_a(a),
                None => inpainted,
            }
            .to_image()?,
            textblocks,
            None,
        ))
    }
}

fn to_mutex<T>(areas: T) -> Arc<parking_lot::Mutex<T>> {
    Arc::new(parking_lot::Mutex::new(areas))
}
