use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MaskRefinementSettings {
    /// The threshold for ignoring text in non bubble areas, with valid values ranging from 1 to 50, does not ignore others. Recommendation 5 to 10. If it is too low, normal bubble areas may be ignored, and if it is too large, non bubble areas may be considered normal bubbles
    pub ignore_bubble: u8,
    /// By how much to extend the text mask to remove left-over text pixels of the original image.
    /// bigger means the mask gets extended further
    pub dilation_offset: f64,
    /// Set the convolution kernel size of the text erasure area to completely clean up text residues
    /// bigger means the mask gets extended further
    pub kernel_size: u8,
    /// Mask gets extended to the right/top to cover furigana characters
    pub furigana: bool,
    pub method: mask_refinement::Method,
}

impl Default for MaskRefinementSettings {
    fn default() -> Self {
        Self {
            method: mask_refinement::Method::FitText,
            ignore_bubble: 0,
            kernel_size: 3,
            dilation_offset: 20.0,
            furigana: false,
        }
    }
}
