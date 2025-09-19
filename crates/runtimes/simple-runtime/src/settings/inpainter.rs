use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(
    Serialize, Deserialize, Default, EnumIter, Hash, PartialEq, Eq, Copy, Clone, JsonSchema,
)]
pub enum Inpainter {
    #[default]
    LamaAot,
    LamaLarge,
    LamaMpe,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct InpainterSettings {
    /// Inpainting model to use
    pub inpainter: Inpainter,
    /// Size of image used for inpainting (too large will result in OOM)
    pub inpainting_size: u16,
    /// If no ai is used for inpainting than use this color
    pub inpaint_color: [u8; 3],
}

impl Default for InpainterSettings {
    fn default() -> Self {
        Self {
            inpainter: Default::default(),
            inpainting_size: 2048,
            inpaint_color: [255; 3],
        }
    }
}
