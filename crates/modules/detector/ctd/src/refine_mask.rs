use interface::detectors::{textlines::Quadrilateral, Mask};
use ndarray::Array2;
use roots::find_roots_quadratic;

fn refine_mask(mask: Mask, blk_list: Vec<Quadrilateral>) -> Mask {
    let mask_refined = Mask {
        data: vec![0; mask.width as usize * mask.height as usize],
        width: mask.width,
        height: mask.height,
    };
    for blk in blk_list {
        let (bx1, by1, bx2, by2) = enlarge_window(blk.xyxy(), img.shape[1], img.shape[0], 2.5, 1.0);
        // let im = img[by1: by2, bx1: bx2];
        // let msk = pred_mask[by1: by2, bx1: bx2];
        // if len(im.shape) == 3 and im.shape[-1] == 3:
        //   im = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)

        let mask_list = get_topk_masklist(im, msk);
    }
    todo!()
}

fn get_topk_masklist(im_grey: Array2<u8>, ped_mask: Array2<u8>) {
    //     candidate_grey_px = im_grey[np.where(cv2.erode(msk, np.ones((3,3), np.uint8), iterations=1) > 127)]
    //     bin, his = np.histogram(candidate_grey_px, bins=255)
}

// def get_topk_masklist(im_grey, pred_mask):

//     topk_color = get_topk_color(his, bin, color_var=10, k=3)
//     color_range = 30
//     mask_list = list()
//     for ii, color in enumerate(topk_color):
//         c_top = min(color+color_range, 255)
//         c_bottom = c_top - 2 * color_range
//         threshed = cv2.inRange(im_grey, c_bottom, c_top)
//         threshed, xor_sum = minxor_thresh(threshed, msk)
//         mask_list.append([threshed, xor_sum])
//     return mask_list
//

//
// def refine_mask(img: np.ndarray, pred_mask: np.ndarray, blk_list: List[Quadrilateral], refine_mode: int = REFINEMASK_INPAINT) -> np.ndarray:
//

//
//         mask_list += get_otsuthresh_masklist(im, msk, per_channel=False)
//         mask_merged = merge_mask_list(mask_list, msk, blk=blk, text_window=[bx1, by1, bx2, by2], refine_mode=refine_mode)
//         mask_refined[by1: by2, bx1: bx2] = cv2.bitwise_or(mask_refined[by1: by2, bx1: bx2], mask_merged)

//     return mask_refined

fn enlarge_window(
    (x1, y1, x2, y2): (i64, i64, i64, i64),
    im_w: u16,
    im_h: u16,
    ratio: f32,
    aspect_ratio: f32,
) -> (i64, i64, i64, i64) {
    assert!(ratio > 1.0);
    let w = x2 - x1;
    let h = y2 - y1;
    // https://numpy.org/doc/stable/reference/generated/numpy.roots.html

    let roots = find_roots_quadratic(
        aspect_ratio,
        w as f32 + h as f32 * aspect_ratio,
        (1.0 - ratio) * w as f32 * h as f32,
    );

    let max = match roots {
        roots::Roots::No(_) => None,
        roots::Roots::One(one) => Some(one[0]),
        roots::Roots::Two(two) => {
            let f = two[0];
            let s = two[1];

            Some(match f < s {
                true => s,
                false => f,
            })
        }
        roots::Roots::Three(t) => t.iter().cloned().fold(None, |max, x| match max {
            None => Some(x),
            Some(y) => Some(if x.partial_cmp(&y) == Some(std::cmp::Ordering::Greater) {
                x
            } else {
                y
            }),
        }),
        roots::Roots::Four(f) => f.iter().cloned().fold(None, |max, x| match max {
            None => Some(x),
            Some(y) => Some(if x.partial_cmp(&y) == Some(std::cmp::Ordering::Greater) {
                x
            } else {
                y
            }),
        }),
    };
    let max = max.unwrap();
    let delta = (max / 2.0).round() as i64;
    let delta_w = (delta as f32 * aspect_ratio) as i64;
    let delta_w = i64::min(x1.min(delta_w), im_w as i64 - x2);
    let delta = i64::min(y1.min(delta), im_h as i64 - y2);
    (x1 - delta_w, y1 - delta, x2 + delta_w, y2 + delta)
}
