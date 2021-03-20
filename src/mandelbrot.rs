/// Needed values to convert image coords to points in complex plane
pub struct Region {
    img_w: u32,
    img_h: u32,
    real_min: Float,
    real_max: Float,
    im_min: Float,
    im_max: Float,
}

pub enum Output {
    InSet,
    NotInSet(usize),
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
pub fn compute(r: &Region, iterations: usize, img_x: u32, img_y: u32) -> Output {
    let x = scale_convert(img_x, 0, r.img_w, r.real_min, r.real_max);
    let y = scale_convert(img_y, 0, r.img_h, r.im_min, r.im_max);

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

        if zr * zr + zi * zi > 4.0 {
            return Output::NotInSet(i);
        }
    }

    InSet
}

