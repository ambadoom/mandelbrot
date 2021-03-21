use rayon::prelude::*;

pub type Float = f64;

const ESCAPE: Float = (1<<16) as Float;
pub const IN_SET: Float = -1.0;

/// Needed values to convert image coords to points in complex plane
pub struct Region {
    pub img_w: u32,
    pub img_h: u32,
    pub real_min: Float,
    pub real_max: Float,
    pub im_min: Float,
    pub im_max: Float,
}

/// Convert a u32 between imn and imx to a Float beween omn and omx
#[inline]
fn scale_convert(i: u32, imn: u32, imx: u32, omn: Float, omx: Float) -> Float {
    let irange = imx - imn;
    let i01 = (i - imn) as Float / irange as Float;
    let orange = omx - omn;
    i01 * orange + omn
}

/// Compute a particular pixel for the final image
#[inline]
fn do_pixel(r: &Region, iterations: usize, img_x: u32, img_y: u32) -> f64 {
    let x = scale_convert(img_x, 0, r.img_w, r.real_min, r.real_max);
    let y = scale_convert(r.img_h - img_y, 0, r.img_h, r.im_min, r.im_max);

    let mut zr: Float = 0.0;
    let mut zi: Float = 0.0;
    for i in 0..iterations {
        // z = z^2
        let tmp = zr * zr - zi * zi;
        zi = zr * zi * 2.0;
        zr = tmp;
        // z = z + c
        zr += x;
        zi += y;

        if zr * zr + zi * zi > ESCAPE {
            // Calculate adjusted iteration count
            let log_z = (zr * zr + zi * zi).log2();
            let log_2 = (2.0_f64).log2();
            let nu = (log_z / log_2).log2() / log_2;
            let iteration = i as Float + 1.0 - nu;
            return iteration;
        }
    }

    // Negative value signals point is in set
    IN_SET
}

/// Generate the image
pub fn generate<F>(region: &Region, iterations: usize, progress: F) -> Vec<f64>
where F: Fn() + Sync {
    (0..(region.img_w * region.img_h))
        .into_par_iter()
        .map(|i| {
            let x = i % region.img_w;
            let y = i / region.img_w;
            progress();

            do_pixel(&region, iterations, x, y)
        })
        .collect()
}
