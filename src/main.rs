use std::time::Duration;

use image;
use image::{ ImageBuffer };

use clap::Clap;

use pbr::ProgressBar;

type Float = f32;
type Pixel = image::Luma<u8>;

#[derive(Clap)]
#[clap(version="0.1.0", author="Louis Stagg")]
struct Opts {
    /// Where to save the generated image
    #[clap(long, short, default_value="output.png")]
    output: String,

    /// Maximum number of iterations
    #[clap(long, default_value="1000")]
    iterations: usize,

    /// Resolution of the image (only supports square output for now)
    #[clap(long, default_value="1024")]
    resolution: u32,

    /// Display a progress bar
    #[clap(long, short)]
    progress: bool,

    /// Center of image real part
    #[clap(default_value="-0.5")]
    real: Float,

    /// Center of image imaginary part
    #[clap(default_value="0.0")]
    imaginary: Float,

    /// Smaller number = more zoom
    #[clap(default_value="1.5")]
    scale: Float,
}

#[inline]
fn scale_convert(i: u32, imn: u32, imx: u32, omn: Float, omx: Float) -> Float {
    let irange = imx - imn;
    let i01 = (i - imn) as Float / irange as Float;
    let orange = omx - omn;
    i01 * orange + omn
}

#[inline]
fn iteration_to_colour(iteration: usize) -> u8 {
    100u8.saturating_add((iteration*100) as u8)
}

struct Region {
    img_w: u32,
    img_h: u32,
    real_min: Float,
    real_max: Float,
    im_min: Float,
    im_max: Float,
}

#[inline]
fn do_pixel(r: &Region, iterations: usize, img_x: u32, img_y: u32) -> Pixel {
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
            return image::Luma([iteration_to_colour(i)]);
        }
    }

    image::Luma([0u8])
}

fn main() {
    let opts = Opts::parse();
    let region = Region {
        img_w: opts.resolution,
        img_h: opts.resolution,
        real_min: opts.real - opts.scale,
        real_max: opts.real + opts.scale,
        im_min: opts.imaginary - opts.scale,
        im_max: opts.imaginary + opts.scale,
    };

    println!("Generating image...");

    let mut bar = if opts.progress {
        let mut b = ProgressBar::new(region.img_w as u64 * region.img_h as u64);
        b.set_max_refresh_rate(Some(Duration::from_millis(200)));
        Some(b)
    } else {
        None
    };
    
    // Construct a new by repeated calls to the supplied closure.
    let img = ImageBuffer::from_fn(region.img_w, region.img_h, |x, y| {
        if let Some(ref mut bar) = bar {
            bar.inc();
        }

        do_pixel(&region, opts.iterations, x, y)
    });

    println!("Saving image...");
    img.save(opts.output).unwrap();

    println!("Done.");

}
