use core::error;
use std::{clone, env, path::Path, process::exit};

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba, imageops::resize};

/// Convert a full rgb color to the nearest 8 bit color escape code
fn color8(r: u8, g: u8, b: u8) -> usize {
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return (r as usize - 8) * 23 / 247 + 232;
    }

    let ansi_r = r as usize * 6 / 256;
    let ansi_g = g as usize * 6 / 256;
    let ansi_b = b as usize * 6 / 256;

    16 + (36 * ansi_r) + (6 * ansi_g) + ansi_b
}

fn print_pixel_8bit(
    rgb1: (u8, u8, u8),
    rgb2: Option<(u8, u8, u8)>,
    last_rgb1: usize,
    last_rgb2: usize,
) -> (usize, usize) {
    let rgb1 = color8(rgb1.0, rgb1.1, rgb1.2);
    if let Some(rgb2) = rgb2 {
        let rgb2 = color8(rgb2.0, rgb2.1, rgb2.2);
        if last_rgb1 != rgb1 && last_rgb2 != rgb2 {
            print!("\x1b[38;5;{rgb2};48;5;{rgb1}m▄");
        } else if last_rgb1 != rgb1 && last_rgb2 == rgb2 {
            print!("\x1b[48;5;{rgb1}m▄");
        } else if last_rgb1 == rgb1 && last_rgb2 != rgb2 {
            print!("\x1b[38;5;{rgb2}m▄");
        } else {
            print!("▄");
        }
        (rgb1, rgb2)
    } else {
        if last_rgb1 != rgb1 {
            print!("\x1b[49;38;5;{rgb1}m▀");
        } else {
            print!("▀");
        }
        (rgb1, 0)
    }
}

fn print_pixel_full(
    rgb1: (u8, u8, u8),
    rgb2: Option<(u8, u8, u8)>,
    last_rgb1: usize,
    last_rgb2: usize,
) -> (usize, usize) {
    let (r1, g1, b1) = rgb1;
    let rgb1_sum = (r1 as usize) << 16 | (g1 as usize) << 8 | (b1 as usize);
    if let Some(rgb2) = rgb2 {
        let (r2, g2, b2) = rgb2;
        let rgb2_sum = (r2 as usize) << 16 | (g2 as usize) << 8 | (b2 as usize);
        if last_rgb1 != rgb1_sum && last_rgb2 != rgb2_sum {
            print!("\x1b[38;2;{r2};{g2};{b2};48;2;{r1};{g1};{b1}m▄")
        } else if last_rgb1 != rgb1_sum && last_rgb2 == rgb2_sum {
            print!("\x1b[48;2;{r1};{g1};{b1}m▄")
        } else if last_rgb1 == rgb1_sum && last_rgb2 != rgb2_sum {
            print!("\x1b[38;2;{r2};{g2};{b2}m▄")
        } else {
            print!("▄")
        }
        (rgb1_sum, rgb2_sum)
    } else {
        if last_rgb1 != rgb1_sum {
            print!("\x1b[49;38;2;{r1};{g1};{b1}m▀")
        } else {
            print!("▀")
        }
        ((r1 as usize) << 16 | (g1 as usize) << 8 | (b1 as usize), 0)
    }
}

fn load_image(picture: &Path, width: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Load and resize the image
    let Ok(file) = ImageReader::open(&picture) else {
        println!("\x1B[31mImage file not found.\x1B[0m");
        exit(1);
    };
    let Ok(img) = file.decode() else {
        println!("\x1B[31mInvalid image file.\x1B[0m");
        exit(1);
    };

    let height = img.height() * width / img.width();
    if img.dimensions().0 != width {
        let resized = resize(&img, width, height, image::imageops::FilterType::Triangle);
        resized
    } else {
        let Some(converted) = img.as_rgba8() else {
            println!("\x1B[31mInvalid image file.\x1B[0m");
            exit(1);
        };
        converted.clone()
    }
}

fn draw_image(picture: &Path, width: u32, mode_8bit: bool) {
    let processed = load_image(picture, width);

    for y in (0..processed.dimensions().1).step_by(2) {
        let mut last_rgb2: usize = 1 << 63;
        let mut last_rgb1: usize = 1 << 63;
        for x in 0..processed.dimensions().0 {
            // Fetch the pixel rgb
            let pixel = processed.get_pixel(x, y);
            let rgb1 = (pixel.0[0], pixel.0[1], pixel.0[2]);
            let pixel = processed.get_pixel(x, y + 1);
            let rgb2 = if y < processed.dimensions().1 - 1 {
                Some((pixel.0[0], pixel.0[1], pixel.0[2]))
            } else {
                None
            };
            if mode_8bit {
                (last_rgb1, last_rgb2) = print_pixel_8bit(rgb1, rgb2, last_rgb1, last_rgb2);
            } else {
                (last_rgb1, last_rgb2) = print_pixel_full(rgb1, rgb2, last_rgb1, last_rgb2);
            }
        }
        println!("\x1b[0m");
    }
}

fn print_help() {
    println!("Syntax: ./image_to_ansi <image path> <width> <color mode>.\n");
    println!(
        "Image path: A valid path to the image that you want to convert. It can be an absolute or a relative path."
    );
    println!(
        "Width: The width of the final image in pixels/characters. The height of the image will be relative to the width. The aspect ratio is kept."
    );
    println!(
        "Color mode: Must be \"full\" or \"8bit\". The first represents the full RGB color space and the second represents the 8bit color mode of the shell.\nThe 8bit mode creates a much smaller output and is more widely supported. The full color mode is supported on most moderne terminal applications."
    );
}

fn main() {
    let mut args = env::args().skip(1);

    // Get the arguments from argv
    let Some(path) = args.next() else {
        println!("\x1B[31mMissing argument '<path>'.\x1B[0m");
        print_help();
        exit(1);
    };

    let Ok(width) = ({
        let Some(arg) = args.next() else {
            println!("\x1B[31mMissing argument '<width>'.\x1B[0m");
            print_help();
            exit(1);
        };

        arg.parse::<u32>()
    }) else {
        println!(
            "\x1B[31mInvalid value for argument '<width>'. It must be a valide u32 number.\x1B[0m"
        );
        exit(1);
    };

    let Some(mode_8bit) = args.next() else {
        println!("\x1B[31mMissing argument '<color mode>'.\x1B[0m");
        print_help();
        exit(1);
    };

    let mode_8bit = match mode_8bit.as_str() {
        "full" => false,
        "8bit" => true,
        _ => {
            println!(
                "\x1B[31mInvalid value for argument '<color mode>'. It must be 'full' or '8bit'.\x1B[0m"
            );
            print_help();
            exit(1);
        }
    };

    draw_image(Path::new(&path), width, mode_8bit);
}
