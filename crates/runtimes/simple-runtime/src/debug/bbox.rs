use std::path::PathBuf;

use interface_detector::textlines::Quadrilateral;
use interface_image::RawImage;
use opencv::{
    core::{Point, Scalar, Vector},
    imgproc::{polylines, LINE_8},
};

pub fn render_bboxes(img: &RawImage, qu: &[Quadrilateral], path: &PathBuf) -> anyhow::Result<()> {
    let mut img = img.as_opencv_mat()?.clone_pointee();
    for q in qu {
        let pts = q
            .pts()
            .iter()
            .map(|v| Point::new(v.x as i32, v.y as i32))
            .collect::<Vector<Point>>();
        polylines(
            &mut img,
            &pts,
            true,
            Scalar::new(255.0, 0.0, 0.0, 255.0),
            2,
            LINE_8,
            0,
        )?;
    }
    RawImage::try_from(img)?
        .to_image()?
        .save(path.join("1_bboxes_unfiltered.png"))?;
    Ok(())
}
