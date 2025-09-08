use std::sync::Arc;

use interface_image::{DimType, ImageOp, Mask, RawImage};

pub fn resize_keep_aspect(
    mut img: RawImage,
    size: u16,
    img_processor: &Arc<dyn ImageOp + Send + Sync>,
) -> RawImage {
    let ratio = size as f64 / img.width.max(img.height) as f64;
    let new_width = img.width as f64 * ratio;
    let new_height = img.height as f64 * ratio;

    img_processor.resize(
        &mut img,
        new_width as DimType,
        new_height as DimType,
        interface_image::Interpolation::BilinearExact,
    )
}

pub fn resize_keep_aspect_mask(
    mut img: Mask,
    size: u16,
    img_processor: &Arc<dyn ImageOp + Send + Sync>,
) -> Mask {
    let ratio = size as f64 / img.width.max(img.height) as f64;
    let new_width = img.width as f64 * ratio;
    let new_height = img.height as f64 * ratio;

    img_processor.resize_mask(
        &mut img,
        new_width as usize,
        new_height as usize,
        interface_image::Interpolation::BilinearExact,
    )
}

pub fn lama_resize_image(
    mut image: RawImage,
    mut mask: Mask,
    inpainting_size: u16,
    img_processor: &Arc<dyn ImageOp + Send + Sync>,
) -> (RawImage, Mask) {
    let w = image.width;
    let h = image.height;
    if w.max(h) > inpainting_size {
        image = resize_keep_aspect(image, inpainting_size, img_processor);
        mask = resize_keep_aspect_mask(mask, inpainting_size, img_processor);
    }
    (image, mask)
}

pub fn lama_add_border(
    mut image: RawImage,
    mut mask: Mask,
    img_processor: &Arc<dyn ImageOp + Send + Sync>,
) -> (RawImage, Mask, u16, u16) {
    let w = image.width;
    let h = image.height;
    let pad_size = 8;
    let new_h = if h % pad_size != 0 {
        (pad_size - (h % pad_size)) + h
    } else {
        h
    };
    let new_w = if w % pad_size != 0 {
        (pad_size - (w % pad_size)) + w
    } else {
        w
    };

    if new_h != h || new_w != w {
        image = img_processor.add_border_wh(image, new_w, new_h);

        let m = img_processor.add_border_wh(
            RawImage {
                data: mask.data,
                width: mask.width,
                height: mask.height,
                channels: 1,
            },
            new_w,
            new_h,
        );
        mask = Mask {
            data: m.data,
            width: m.width,
            height: m.height,
        };
    }
    (image, mask, new_w, new_h)
}
