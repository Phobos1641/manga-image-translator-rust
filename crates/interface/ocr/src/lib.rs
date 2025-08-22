use interface_detector::textlines::Quadrilateral;
use interface_image::{ImageOp, Mask, RawImage};

#[async_trait::async_trait]
pub trait Ocr {
    async fn detect(
        &mut self,
        image: &RawImage,
        areas: &[Quadrilateral],
        img_processor: &Box<dyn ImageOp + Send + Sync>,
    ) -> anyhow::Result<Vec<QuadrilateralInfo>>;

    /// image is already the sliced image
    async fn detect_patch(
        &mut self,
        sliced_image: Mask,
        area: Quadrilateral,
        img_processor: &Box<dyn ImageOp + Send + Sync>,
    ) -> anyhow::Result<QuadrilateralInfo>;
}

#[derive(Debug, Clone)]
pub struct QuadrilateralInfo {
    pub text: String,
    pub fg: Option<[u8; 3]>,
    pub bg: Option<[u8; 3]>,
    pub pos: Quadrilateral,
}
