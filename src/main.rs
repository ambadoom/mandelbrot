use std::time::{Instant, Duration};
use std::sync::Mutex;

use image::{GrayImage, imageops::FilterType};

use clap::Clap;

use pbr::ProgressBar;

use rayon::prelude::*;

type Float = f64;

/// Command line options
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

    /// Generate at a higher resolution and scale down
    #[clap(long, default_value="2")]
    supersample: u32,

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

/// Needed values to convert image coords to points in complex plane
struct Region {
    img_w: u32,
    img_h: u32,
    real_min: Float,
    real_max: Float,
    im_min: Float,
    im_max: Float,
}

/// Convert a u32 between imn and imx to a Float beween omn and omx
#[inline]
fn scale_convert(i: u32, imn: u32, imx: u32, omn: Float, omx: Float) -> Float {
    let irange = imx - imn;
    let i01 = (i - imn) as Float / irange as Float;
    let orange = omx - omn;
    i01 * orange + omn
}

const ESCAPE: Float = (1<<16) as Float;

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
    -1.0
}

/// Generate the image
fn generate<F>(region: &Region, iterations: usize, progress: F) -> Vec<f64>
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

fn colour(generated: &[f64]) -> Vec<u8> {
    generated
        .iter()
        .map(|i| {
            if *i == -1.0 {
                0
            } else {
                *i as u8
            }
        })
        .collect()
}

fn main() {
    let opts = Opts::parse();
    let region = Region {
        img_w: opts.resolution * opts.supersample,
        img_h: opts.resolution * opts.supersample,
        real_min: opts.real - opts.scale,
        real_max: opts.real + opts.scale,
        im_min: opts.imaginary - opts.scale,
        im_max: opts.imaginary + opts.scale,
    };

    println!("Generating image...");

    let start = Instant::now();

    let generated = if opts.progress {
        let mut b = ProgressBar::new(region.img_w as u64 * region.img_h as u64);
        b.set_max_refresh_rate(Some(Duration::from_millis(200)));
        let mb = Mutex::new(b);
        generate(&region, opts.iterations, || { mb.lock().unwrap().inc(); })
    } else {
        generate(&region, opts.iterations, || {})
    };

    let elapsed = start.elapsed();
    if elapsed.as_millis() > 0 {
        println!(
            "Generated in {}ms ({} pixels/s)",
            elapsed.as_millis(),
            (region.img_w as u64 * region.img_h as u64 * 1000) / elapsed.as_millis() as u64
        );
    } else {
        println!(
            "Generated in {}ms",
            elapsed.as_millis(),
        );
    }

    let pixels = colour(&generated);

    println!("Saving image...");

    let img = GrayImage::from_vec(region.img_w, region.img_h, pixels).unwrap();
    let img_ = image::imageops::resize(&img, opts.resolution, opts.resolution, FilterType::Triangle);
    let result = img_.save(opts.output);
    if let Err(e) = result {
        println!("Failed to save image: {}", e);
        std::process::exit(1);
    }

    println!("Done.");
}
