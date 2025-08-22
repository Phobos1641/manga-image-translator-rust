use std::{fs::File, io::Write, path::PathBuf};

use image::{DynamicImage, GrayImage};
use interface_image::{CpuImageProcessor, ImageOp, RawImage};

use crate::{debug::render_bboxes, dict::Dict, settings::Settings, setup::Models};

impl Models {
    pub async fn execute(
        &mut self,
        img: DynamicImage,
        config: &Settings,
        debug_path: Option<PathBuf>,
    ) {
        let img_processor =
            Box::new(CpuImageProcessor::default()) as Box<dyn ImageOp + Sync + Send>;

        let (mut img, mut alpha) = RawImage::rgba(img);

        if let Some(upscaler) = config.upscaler.upscaler {
            let (h, w) = (img.height, img.width);

            img = self
                .get_upscaler(upscaler)
                .upscale(
                    &img,
                    config.upscaler.patch_size,
                    config.upscaler.padding,
                    &img_processor,
                )
                .unwrap();
            let (ha, wa) = (img.height, img.width);
            if let Some(a) = alpha {
                let alpha_image =
                    DynamicImage::from(GrayImage::from_raw(w as u32, h as u32, a).unwrap());
                let alpha_image = alpha_image.resize_exact(
                    wa as u32,
                    ha as u32,
                    image::imageops::FilterType::Lanczos3,
                );
                let data = match alpha_image {
                    DynamicImage::ImageLuma8(img) => img.into_raw(),
                    _ => unreachable!("the output from upscaling should be a gray image"),
                };
                alpha = Some(data);
            }
        }

        if let Some(debug_path) = &debug_path {
            img.clone()
                .to_image()
                .unwrap()
                .save(debug_path.join("0_input.png"))
                .unwrap();
            File::create(debug_path.join("0_config.json"))
                .unwrap()
                .write_all(serde_json::to_string(&config).unwrap().as_bytes())
                .unwrap();
        }

        let (areas, mask) = self
            .get_detector(config.detector.detector)
            .detect(
                &img,
                config.detector.preprocessor,
                config.detector.options,
                &img_processor,
            )
            .unwrap();

        if let Some(debug_path) = &debug_path {
            mask.clone()
                .to_image()
                .unwrap()
                .save(debug_path.join("1_mask_raw.png"))
                .unwrap();
            render_bboxes(&img, &areas, debug_path);
        }

        let textlines = self
            .get_ocr(config.ocr.ocr)
            .detect(&img, &areas, &img_processor)
            .await
            .unwrap();
        let mut textblocks = textline_merge::dispatch_main(
            &textlines,
            img.width,
            img.height,
            config.ocr.min_text_length,
            vec![],
            &config.ocr.filter_text,
            &self.lang_detector,
        );

        if let Some(pre_dict) = &config.translator.pre_dict {
            //TODO: add caching
            let dict = Dict::try_load(pre_dict);
            for textblock in &mut textblocks {
                textblock.text = dict.apply(&textblock.text);
            }
        }

        let translator = self.get_translator(config.translator.translator);
        let texts = textblocks
            .iter()
            .filter(|v| !v.skip_translate)
            .map(|v| v.text.clone())
            .collect::<Vec<_>>();
        let translations = if translator.local() {
            translator
                .translator_mut()
                .as_blocking()
                .unwrap()
                .translate_vec(
                    &texts,
                    None,
                    interface_translator::Language::Japanese,
                    &interface_translator::Language::English,
                )
                .unwrap()
        } else {
            translator
                .translator()
                .as_async()
                .unwrap()
                .translate_vec(&texts, None, None, &interface_translator::Language::English)
                .await
                .unwrap()
                .text
        };

        let mask_refined = mask_refinement::dispatch(
            &textblocks,
            &img,
            &mask,
            mask_refinement::Method::FillMask,
            5,
            true,
            &img_processor,
        );

        if let Some(debug_path) = &debug_path {
            mask_refined
                .to_image()
                .unwrap()
                .save(debug_path.join("4_mask_refined.png"))
                .unwrap();
        }
        // inpainting
        // rgb => rgba
        // rendering
    }
}
