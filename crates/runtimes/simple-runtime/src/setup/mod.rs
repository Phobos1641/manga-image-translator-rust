mod detector;
mod inpainter;
mod ocr;
mod translator;
mod upscaler;
pub use detector::DetectorType;
pub use detector::Detectors;
use interface_translator::LangIdDetector;
pub use upscaler::UpscalerType;
pub use upscaler::Upscalers;

use crate::settings::Detector;
use crate::settings::Inpainter;
use crate::settings::Translator;
use crate::settings::Upscaler;
use crate::settings::OCR;
use crate::setup::inpainter::InpainterType;
use crate::setup::inpainter::Inpainters;
use crate::setup::ocr::OCRs;
use crate::setup::ocr::OcrType;
use crate::setup::translator::TranslatorType;
use crate::setup::translator::Translators;

pub struct Models {
    upscalers: Upscalers,
    detectors: Detectors,
    ocrs: OCRs,
    translators: Translators,
    inpainters: Inpainters,
    pub lang_detector: LangIdDetector,
}

impl Models {
    pub fn get_upscaler(&mut self, upscaler: Upscaler) -> &mut UpscalerType {
        self.upscalers.get(upscaler)
    }
    pub fn get_detector(&mut self, detector: Detector) -> &mut DetectorType {
        self.detectors.get(detector)
    }
    pub fn get_ocr(&mut self, ocr: OCR) -> &mut OcrType {
        self.ocrs.get(ocr)
    }
    pub fn get_translator(&mut self, translator: Translator) -> &mut TranslatorType {
        self.translators.get(translator)
    }
    pub fn get_inpainter(&mut self, inpainter: Inpainter) -> &mut InpainterType {
        self.inpainters.get(inpainter)
    }
    pub async fn new(
        max_batch_size_upscaler: usize,
        max_batch_size_ocr: usize,
        fast: bool,
        cuda: bool,
    ) -> Self {
        //TODO: providers based on input
        Models {
            lang_detector: LangIdDetector::new().unwrap(),
            detectors: Detectors::new(),
            upscalers: Upscalers::new(max_batch_size_upscaler, fast),
            inpainters: Inpainters::new(),
            ocrs: OCRs::new(max_batch_size_ocr),
            translators: Translators::new(cuda).await,
        }
    }
}
