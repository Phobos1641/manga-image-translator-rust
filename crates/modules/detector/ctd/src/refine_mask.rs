use imageproc::{distance_transform::Norm, image::GrayImage, morphology::erode};

use interface_detector::textlines::Quadrilateral;
use interface_image::{Mask, RawImage};
use ndarray::{Array2, Zip};
use roots::find_roots_quadratic;

fn refine_mask(img: RawImage, mask: Mask, blk_list: Vec<Quadrilateral>) -> Mask {
    let mask_refined = Mask {
        data: vec![0; mask.width as usize * mask.height as usize],
        width: mask.width,
        height: mask.height,
    };
    for blk in blk_list {
        let (bx1, by1, bx2, by2) = enlarge_window(blk.xyxy(), img.width, img.height, 2.5, 1.0);
        // let im = img[by1: by2, bx1: bx2];
        // let msk = pred_mask[by1: by2, bx1: bx2];
        // if len(im.shape) == 3 and im.shape[-1] == 3:
        //   im = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)

        let mask_list = get_topk_masklist(im, msk);
        mask_list.extend(get_otsuthresh_masklist(&img, msk));
        // mask_merged = merge_mask_list(mask_list, msk, blk=blk, text_window=[bx1, by1, bx2, by2], refine_mode=refine_mode)
        // mask_refined[by1: by2, bx1: bx2] = cv2.bitwise_or(mask_refined[by1: by2, bx1: bx2], mask_merged)
    }
    mask_refined
}

fn ndarray_to_gray_image(arr: &Array2<u8>) -> GrayImage {
    let data = arr
        .as_slice()
        .map(|v| v.to_vec())
        .unwrap_or_else(|| arr.clone().into_iter().collect());
    GrayImage::from_raw(arr.dim().1 as u32, arr.dim().0 as u32, data).unwrap()
}

fn gray_image_to_ndarray(img: &GrayImage) -> Array2<u8> {
    todo!()
}

fn extract_candidates(im_grey: &Array2<u8>, mask: &Array2<u8>) -> Vec<u8> {
    let mask_img = ndarray_to_gray_image(mask);
    let eroded_img = erode(&mask_img, Norm::LInf, 1);
    let eroded_mask = gray_image_to_ndarray(&eroded_img);
    let mut result = Vec::new();
    for ((y, x), &val) in eroded_mask.indexed_iter() {
        if val > 127 {
            result.push(im_grey[(y, x)]);
        }
    }
    result
}

fn get_topk_masklist(im_grey: Array2<u8>, ped_mask: Array2<u8>) -> Vec<(Array2<u8>, u64)> {
    let candidate_grey_px = extract_candidates(&im_grey, &ped_mask);
    let (bin, his) = histogram(&candidate_grey_px, 255);
    let topk_color = get_topk_color(his, bin, 3, 10, 0.001);
    let color_range = 30;
    topk_color
        .into_iter()
        .enumerate()
        .map(|(ii, color)| {
            let c_top = 255.min(color + color_range);
            let c_bottom = c_top - 2 * color_range;
            let threshed = cv2.inRange(im_grey, c_bottom, c_top);
            let (threshed, xor_sum) = minxor_thresh(threshed, &ped_mask, false);
            (threshed, xor_sum)
        })
        .collect()
}

fn argsort_descending(v: &[i32]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..v.len()).collect();
    indices.sort_by_key(|&i| -v[i]);
    indices
}
fn get_topk_color(
    color_list: Vec<u32>,
    bins: Vec<usize>,
    k: usize,
    color_var: u32,
    bin_tol: f64,
) -> Vec<u32> {
    let idx = argsort_descending(&bins.iter().map(|v| *v as i32).collect::<Vec<_>>());
    let mut color_list = idx.iter().map(|v| color_list[*v]);
    let bins = idx.iter().map(|v| bins[*v]).collect::<Vec<_>>();
    let mut top_colors = vec![color_list.next().unwrap()];
    let bin_tol = bins.iter().sum::<usize>() as f64 * bin_tol;
    for (color, bin) in color_list.zip(bins.iter()) {
        if let Some(v) = top_colors.iter().map(|v| (*v) as i32 - color as i32).min() {
            let v = if v < 0 { (v * -1) as u32 } else { v as u32 };
            if v > color_var {
                top_colors.push(color);
            }
        }
        if top_colors.len() >= k || (*bin as f64) < bin_tol {
            break;
        }
    }
    top_colors
}

fn histogram(candidate_grey_px: &[u8], bins: usize) -> (Vec<usize>, Vec<u32>) {
    assert!(bins <= 256, "bins should be ≤ 256 for u8 input");

    let mut hist = vec![0u32; bins];
    for &val in candidate_grey_px {
        let bin = (val as usize * bins) / 256;
        if bin < bins {
            hist[bin] += 1;
        }
    }

    let bin_edges: Vec<usize> = (0..=bins).map(|b| (b * 256) / bins).collect();

    (bin_edges, hist)
}

// def merge_mask_list(mask_list, pred_mask, blk: Quadrilateral = None, pred_thresh=30, text_window=None, filter_with_lines=False, refine_mode=REFINEMASK_INPAINT):
//     mask_list.sort(key=lambda x: x[1])
//     linemask = None
//     if blk is not None and filter_with_lines:
//         linemask = np.zeros_like(pred_mask)
//         lines = blk.pts.astype(np.int64)
//         for line in lines:
//             line[..., 0] -= text_window[0]
//             line[..., 1] -= text_window[1]
//             cv2.fillPoly(linemask, [line], 255)
//         linemask = cv2.dilate(linemask, np.ones((3, 3), np.uint8), iterations=3)

//     if pred_thresh > 0:
//         e_size = 1
//         element = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * e_size + 1, 2 * e_size + 1),(e_size, e_size))
//         pred_mask = cv2.erode(pred_mask, element, iterations=1)
//         _, pred_mask = cv2.threshold(pred_mask, 60, 255, cv2.THRESH_BINARY)
//     connectivity = 8
//     mask_merged = np.zeros_like(pred_mask)
//     for ii, (candidate_mask, xor_sum) in enumerate(mask_list):
//         num_labels, labels, stats, centroids = cv2.connectedComponentsWithStats(candidate_mask, connectivity, cv2.CV_16U)
//         for label_index, stat, centroid in zip(range(num_labels), stats, centroids):
//             if label_index != 0: # skip background label
//                 x, y, w, h, area = stat
//                 if w * h < 3:
//                     continue
//                 x1, y1, x2, y2 = x, y, x+w, y+h
//                 label_local = labels[y1: y2, x1: x2]
//                 label_coordinates = np.where(label_local==label_index)
//                 tmp_merged = np.zeros_like(label_local, np.uint8)
//                 tmp_merged[label_coordinates] = 255
//                 tmp_merged = cv2.bitwise_or(mask_merged[y1: y2, x1: x2], tmp_merged)
//                 xor_merged = cv2.bitwise_xor(tmp_merged, pred_mask[y1: y2, x1: x2]).sum()
//                 xor_origin = cv2.bitwise_xor(mask_merged[y1: y2, x1: x2], pred_mask[y1: y2, x1: x2]).sum()
//                 if xor_merged < xor_origin:
//                     mask_merged[y1: y2, x1: x2] = tmp_merged

//     if refine_mode == REFINEMASK_INPAINT:
//         mask_merged = cv2.dilate(mask_merged, np.ones((5, 5), np.uint8), iterations=1)
//     # fill holes
//     num_labels, labels, stats, centroids = cv2.connectedComponentsWithStats(255-mask_merged, connectivity, cv2.CV_16U)
//     sorted_area = np.sort(stats[:, -1])
//     if len(sorted_area) > 1:
//         area_thresh = sorted_area[-2]
//     else:
//         area_thresh = sorted_area[-1]
//     for label_index, stat, centroid in zip(range(num_labels), stats, centroids):
//         x, y, w, h, area = stat
//         if area < area_thresh:
//             x1, y1, x2, y2 = x, y, x+w, y+h
//             label_local = labels[y1: y2, x1: x2]
//             label_coordinates = np.where(label_local==label_index)
//             tmp_merged = np.zeros_like(label_local, np.uint8)
//             tmp_merged[label_coordinates] = 255
//             tmp_merged = cv2.bitwise_or(mask_merged[y1: y2, x1: x2], tmp_merged)
//             xor_merged = cv2.bitwise_xor(tmp_merged, pred_mask[y1: y2, x1: x2]).sum()
//             xor_origin = cv2.bitwise_xor(mask_merged[y1: y2, x1: x2], pred_mask[y1: y2, x1: x2]).sum()
//             if xor_merged < xor_origin:
//                 mask_merged[y1: y2, x1: x2] = tmp_merged
//     return mask_merged

fn get_otsuthresh_masklist(img: &RawImage, pred_mask: Array2<u8>) -> Vec<(Array2<u8>, u64)> {
    let channels = img.channels();
    let mask_list = channels
        .into_iter()
        .map(|c| {
            let (_, threshed) = cv2.threshold(c, 1, 255, cv2.THRESH_OTSU + cv2.THRESH_BINARY);
            let (threshed, xor_sum) = minxor_thresh(threshed, &pred_mask, false);
            (threshed, xor_sum)
        })
        .collect::<Vec<_>>();
    vec![mask_list.into_iter().min_by_key(|v| v.1).unwrap()]
}

fn minxor_thresh(threshed: Array2<u8>, mask: &Array2<u8>, dilate: bool) -> (Array2<u8>, u64) {
    let neg_threshed = threshed.clone().mapv(|v| 255 - v);
    if dilate {
        // let e_size = 1;
        // element = cv2.getStructuringElement(cv2.MORPH_RECT, (2 * e_size + 1, 2 * e_size + 1),(e_size, e_size))
        // neg_threshed = cv2.dilate(neg_threshed, element, iterations=1)
        // threshed = cv2.dilate(threshed, element, iterations=1)
        unimplemented!()
    }
    let neg_xor_sum = Zip::from(&neg_threshed)
        .and(mask)
        .fold(0u64, |acc, &x, &y| acc + (x ^ y) as u64);
    let xor_sum = Zip::from(&threshed)
        .and(mask)
        .fold(0u64, |acc, &x, &y| acc + (x ^ y) as u64);
    if neg_xor_sum < xor_sum {
        return (neg_threshed, neg_xor_sum);
    } else {
        return (threshed, xor_sum);
    }
}

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
