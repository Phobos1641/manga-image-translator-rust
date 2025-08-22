use std::{io::Cursor, ops::Deref};

use image::{
    write_buffer_with_format, DynamicImage, EncodableLayout, GenericImageView, ImageBuffer,
    ImageFormat, Pixel, PixelWithColorType,
};
use textline_merge::TextBlock;
pub struct Export {
    img: Image,
    patches: Vec<Patch>,
}

pub fn convert<P: Pixel + PixelWithColorType, Container>(
    img: &ImageBuffer<P, Container>,
    format: ImageFormat,
) -> Vec<u8>
where
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]>,
{
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let buf = &img.as_raw()[..(img.width() * img.height()) as usize];
    write_buffer_with_format(
        &mut cursor,
        buf.as_bytes(),
        img.width(),
        img.height(),
        <P as PixelWithColorType>::COLOR_TYPE,
        format,
    )
    .unwrap();

    cursor.into_inner()
}

impl Export {
    pub fn new(
        raw: DynamicImage,
        inpainted: DynamicImage,
        blocks: Vec<TextBlock>,
        format: Option<ImageFormat>,
    ) -> Self {
        let mut patches = Vec::new();
        for block in blocks {
            let xyxy = block.xyxy();
            let patch = inpainted.view(
                xyxy.0 as u32,
                xyxy.1 as u32,
                xyxy.2 as u32 - xyxy.0 as u32,
                xyxy.3 as u32 - xyxy.1 as u32,
            );
            let data = match format {
                Some(format) => convert(&patch.to_image(), format),
                None => patch.to_image().as_bytes().to_vec(),
            };

            patches.push(Patch {
                info: block,
                pos: (xyxy.0 as usize, xyxy.1 as usize),
                bg: Image {
                    width: patch.width() as u16,
                    height: patch.height() as u16,
                    data,
                    raw: format.is_none(),
                },
            });
        }

        let raw_data = match format {
            Some(format) => match &raw {
                DynamicImage::ImageLuma8(img) => convert(img, format),
                DynamicImage::ImageLumaA8(img) => convert(img, format),
                DynamicImage::ImageRgb8(img) => convert(img, format),
                DynamicImage::ImageRgba8(img) => convert(img, format),
                DynamicImage::ImageLuma16(img) => convert(img, format),
                DynamicImage::ImageLumaA16(img) => convert(img, format),
                DynamicImage::ImageRgb16(img) => convert(img, format),
                DynamicImage::ImageRgba16(img) => convert(img, format),
                DynamicImage::ImageRgb32F(img) => convert(img, format),
                DynamicImage::ImageRgba32F(img) => convert(img, format),
                _ => unimplemented!("not implemented yet"),
            },
            None => raw.as_bytes().to_vec(),
        };
        Self {
            img: Image {
                width: raw.width() as u16,
                height: raw.height() as u16,
                data: raw_data,
                raw: format.is_none(),
            },
            patches,
        }
    }
}

pub struct Image {
    width: u16,
    height: u16,
    data: Vec<u8>,
    raw: bool,
}

impl Image {
    pub fn new(width: u16, height: u16, data: Vec<u8>, raw: bool) -> Self {
        Image {
            width,
            height,
            data,
            raw,
        }
    }
}
pub struct Obb {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    rotation: u16,
}
pub struct Point {
    x: usize,
    y: usize,
}

pub struct Patch {
    info: TextBlock,
    pos: (usize, usize),
    bg: Image,
}
