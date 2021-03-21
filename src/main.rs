use std::time::{Instant, Duration};
use std::sync::Mutex;

use image::{GrayImage, RgbImage, imageops::FilterType};

use clap::Clap;

use pbr::ProgressBar;

use mandelbrot::{Region, Float, generate, IN_SET};

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

    /// Disable colour in output
    #[clap(long)]
    grayscale: bool,

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

fn colour_l(generated: &[f64]) -> Vec<u8> {
    generated
        .iter()
        .map(|i| {
            if *i == IN_SET {
                0
            } else {
                *i as u8
            }
        })
        .collect()
}

fn colour_rgb(generated: &[f64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(generated.len() * 3);
    for i in generated.iter() {
        if *i == IN_SET {
            out.extend(&[0, 0, 0])
        } else {
            let a = *i;
            out.extend(&[0, 0, a as u8])
        }
    }
    out
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

    println!("Saving image...");

    if opts.grayscale {
        let img = GrayImage::from_vec(
            region.img_w,
            region.img_h,
            colour_l(&generated)
        ).unwrap();
        let img_ = image::imageops::resize(
            &img,
            opts.resolution,
            opts.resolution,
            FilterType::Triangle
        );
        let result = img_.save(opts.output);
        if let Err(e) = result {
            println!("Failed to save image: {}", e);
            std::process::exit(1);
        }
    } else {
        let img = RgbImage::from_vec(
            region.img_w,
            region.img_h,
            colour_rgb(&generated)
        ).unwrap();
        let img_ = image::imageops::resize(
            &img,
            opts.resolution,
            opts.resolution,
            FilterType::Triangle
        );
        let result = img_.save(opts.output);
        if let Err(e) = result {
            println!("Failed to save image: {}", e);
            std::process::exit(1);
        }
    }

    println!("Done.");
}
