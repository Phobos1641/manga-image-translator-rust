use ndarray::Array2;

unsafe extern "C" {
    unsafe fn run_densecrf(
        unary: *const f32,
        width: i32,
        height: i32,
        n_classes: i32,
        image: *const u8,
        num_iterations: i32,
        out_probs: *mut f32,
    );
}

pub fn densecrf(
    unary: &[f32],
    width: u32,
    height: u32,
    n_classes: u32,
    image: &[u8],
    iterations: i32,
) -> Array2<f32> {
    let width = width as i32;
    let height = height as i32;
    let n_classes = n_classes as i32;
    let num_pixels = width * height;
    let mut out_probs = vec![0.0f32; num_pixels as usize * n_classes as usize];
    unsafe {
        run_densecrf(
            unary.as_ptr(),
            width,
            height,
            n_classes,
            image.as_ptr(),
            iterations,
            out_probs.as_mut_ptr(),
        );
    };
    Array2::from_shape_vec(
        (n_classes as usize, width as usize * height as usize),
        out_probs,
    )
    .unwrap()
}
