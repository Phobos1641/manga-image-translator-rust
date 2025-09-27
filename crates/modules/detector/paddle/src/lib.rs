use std::{f32, sync::Arc};

use base_util::{
    onnx::{new_session, Providers},
    RawSerializable,
};
use geo::{MinimumRotatedRect, Point};

use interface_detector::{
    textlines::{MyPoint, Quadrilateral},
    DefaultOptions, Detector,
};
use interface_image::{ImageOp, Interpolation, Mask, RawImageCow};
use interface_model::{
    impl_model_helpers, impl_model_load_helpers, Model, ModelLoad, ModelRead, ModelSource,
    ModelWrap,
};
use maplit::hashmap;
use ort::{inputs, session::RunOptions, value::Tensor};
use ort_parallel::AsyncSessionPool;
use paddle_ocr_rs::{
    db_net::{DbNet, MEAN_VALUES, NORM_VALUES},
    scale_param::ScaleParam,
};

pub struct PaddleDetector {
    db_net: ModelWrap<AsyncSessionPool>,
    providers: Arc<Vec<Providers>>,
}

impl PaddleDetector {
    ///convnext: Different model architecture, but based on dbnet
    pub fn new(providers: Arc<Vec<Providers>>) -> Self {
        PaddleDetector {
            providers,
            db_net: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl ModelLoad for PaddleDetector {
    impl_model_load_helpers!(db_net, AsyncSessionPool);

    async fn reload(&self) -> anyhow::Result<ModelRead<'_, Self::T>> {
        let model = self.download_model("det", "det.onnx").await?;
        let b = new_session(&self.providers)?;
        let p = AsyncSessionPool::commit_from_file(b, &model, 10)?;

        *self.db_net.write().await = Some(p);
        Ok(self.get_model().await.expect("set before"))
    }
}

impl Model for PaddleDetector {
    impl_model_helpers!("detector", "paddle", db_net);

    fn models(&self) -> std::collections::HashMap<&'static str, ModelSource> {
        hashmap! {
            "det" => ModelSource { url: "https://github.com/frederik-uni/manga-image-translator-rust/releases/download/paddle-ocr-chinese-v4/det.onnx", hash: "b21a993484b367c0ea29d4a703c038d6ee3212173e6abf962b09188b032a9483" },
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PaddleOptions {
    /// How much to extend text skeleton to form bounding box
    /// smaller values = smaller text skeleton.
    /// to small = more false negatives/partial detections
    /// larger values = bigger text skeleton detections .
    /// to big =  more false positives/Multiple close text lines/words may be merged
    /// Suggested values:
    /// - `1.0 – 1.5`: Use for tight text layouts, well-separated characters or lines, high-resolution images.
    /// - `1.5 – 2.0`: General-purpose setting. Provides a good balance between recall and precision.
    /// - `2.0 – 2.5`: Use when text is thin, faint, or sparse—e.g., scanned documents or light fonts.
    /// - `> 2.5`: Rarely needed. May cause nearby text instances to merge or overlap.
    pub unclip_ratio: f64,
    /// Threshold for text detection
    /// smaller values = more detections + more false positives
    /// larger values = fewer detections + more false negatives
    /// allowed range is from 0.0 to 1.0
    pub text_threshold: f64,
    /// Threshold for bbox generation
    /// to small = more false positives/ noise, background artifacts, or partial text.
    /// to big = false negatives/ actual text that had slightly lower confidence is discarded.
    /// allowed range is from 0.0 to 1.0
    pub box_threshold: f64,
}

impl RawSerializable for PaddleOptions {}

impl Default for PaddleOptions {
    fn default() -> Self {
        Self {
            unclip_ratio: 2.3,
            text_threshold: 0.5,
            box_threshold: 0.7,
        }
    }
}

#[async_trait::async_trait]
impl Detector for PaddleDetector {
    async fn infer(
        &self,
        img: RawImageCow<'_>,
        options: DefaultOptions,
        image_op: &Arc<dyn ImageOp + Send + Sync>,
    ) -> anyhow::Result<(Vec<Quadrilateral>, Mask)> {
        let session = self.load().await?;
        let img = img.view();

        let max_side_len = 960;
        let origin_max_side = img.width.max(img.height);

        let resize = if max_side_len <= 0 || max_side_len > origin_max_side {
            origin_max_side
        } else {
            max_side_len
        };
        // resize += 2 * padding;

        // let padding_src = OcrUtils::make_padding(img_src, padding).unwrap();
        let (w, h) = (img.width, img.height);
        let scale = ScaleParam::get_scale_param_size(w as u32, h as u32, resize as u32);
        let src_resize = image_op.resize(
            img,
            scale.dst_width as u16,
            scale.dst_height as u16,
            Interpolation::Bilinear,
        )?;

        let op = RunOptions::new()?;

        let input_tensors = image_op.substract_mean_normalize(
            &src_resize,
            MEAN_VALUES.as_slice(),
            NORM_VALUES.as_slice(),
        );

        let tensor = Tensor::from_array(input_tensors)?;

        let outputs = session
            .run_async(inputs![session.inputs[0].name.clone() => tensor], &op)
            .await?;

        let text_boxes = DbNet::get_text_boxes_core(
            &outputs,
            src_resize.height as u32,
            src_resize.width as u32,
            &ScaleParam::new(
                scale.src_width,
                scale.src_height,
                scale.dst_width,
                scale.dst_height,
                scale.scale_width,
                scale.scale_height,
            ),
            options.text_threshold as f32,
            options.box_threshold as f32,
            options.unclip_ratio as f32,
        )?;

        let boxes = text_boxes
            .into_iter()
            .filter(|v| v.score != f32::INFINITY)
            .map(|v| {
                let pts = v
                    .points
                    .into_iter()
                    .map(|v| (v.x as i64, v.y as i64))
                    .collect::<Vec<_>>();
                let poly = Quadrilateral::new(pts, 0.0).polygon();
                let corners: Vec<Point> = poly
                    .minimum_rotated_rect()
                    .unwrap()
                    .exterior()
                    .points()
                    .take(4)
                    .collect();
                let rolled: Vec<_> = corners
                    .into_iter()
                    .cycle()
                    .skip(2)
                    .take(4)
                    .map(|v| (v.0.x as i64, v.0.y as i64))
                    .collect();
                Quadrilateral::new(rolled, v.score as f64)
            })
            .collect::<Vec<_>>();

        let area = fill_polys_mask(
            boxes.iter().map(|v| v.pts()).collect(),
            w as usize,
            h as usize,
        );

        let mask = Mask {
            width: w,
            height: h,
            data: area,
        };

        Ok((boxes, mask))
    }
}

pub fn fill_polys_mask(pts: Vec<&[MyPoint; 4]>, width: usize, height: usize) -> Vec<u8> {
    let mut mask = vec![0u8; width * height];

    for quad in pts {
        fill_polygon(&mut mask, width, height, quad);
    }

    mask
}
fn fill_polygon(mask: &mut [u8], width: usize, height: usize, poly: &[MyPoint; 4]) {
    let mut edges = Vec::new();

    for i in 0..4 {
        let point = poly[i];
        let point2 = poly[(i + 1) % 4];
        if point.y != point2.y {
            edges.push(((point.x, point.y), (point2.x, point2.y)));
        }
    }

    let min_y = poly.iter().map(|point| point.y).min().unwrap().max(0) as usize;
    let max_y = poly
        .iter()
        .map(|point| point.y)
        .max()
        .unwrap()
        .min(height as i64 - 1) as usize;

    for y in min_y..=max_y {
        let y = y as i64;
        let mut x_intersections = vec![];

        for &((x0, y0), (x1, y1)) in &edges {
            if (y0 <= y && y < y1) || (y1 <= y && y < y0) {
                let dy = y1 - y0;
                let dx = x1 - x0;
                let t = (y - y0) as f64 / dy as f64;
                let x = x0 as f64 + t * dx as f64;
                x_intersections.push(x as i64);
            }
        }

        x_intersections.sort_unstable();
        for pair in x_intersections.chunks(2) {
            if let [x_start, x_end] = *pair {
                let x0 = x_start.max(0).min(width as i64 - 1) as usize;
                let x1 = x_end.max(0).min(width as i64 - 1) as usize;
                for x in x0..=x1 {
                    mask[y as usize * width + x] = 255;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::PaddleDetector;
    use base_util::onnx::all_providers;
    use interface_detector::{DefaultOptions, Detector as _, PreprocessorOptions};
    use interface_image::{CpuImageProcessor, ImageOp, RawImage};
    use interface_model::{Model as _, ModelLoad};

    #[tokio::test]
    async fn load_unload() {
        let data = PaddleDetector::new(Arc::new(all_providers()));
        data.load().await.expect("failed to load model");
        data.unload().await;
    }

    #[tokio::test]
    async fn run() {
        let data = PaddleDetector::new(Arc::new(all_providers()));
        let cpu_image_processor =
            Arc::new(CpuImageProcessor::default()) as Arc<dyn ImageOp + Send + Sync>;
        data.load().await.expect("Failed to load data");
        data.detect(
            &RawImage::new("./imgs/232265329-6a560438-e887-4f7f-b6a1-a61b8648f781.png")
                .expect("Failed to load image"),
            PreprocessorOptions::default(),
            DefaultOptions::default(),
            &cpu_image_processor,
        )
        .await
        .expect("failed to detect");
    }
}
