use std::{env, path::Path};

use image::{GenericImageView, ImageReader, imageops::resize};


/// Convert a full rgb color to the nearest 8 bit color escape code
fn color8(r: usize, g: usize, b: usize) -> usize {
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return (r - 8) * 23 / 247 + 232;
    }

    let ansi_r = r * 6 / 256;
    let ansi_g = g * 6 / 256;
    let ansi_b = b * 6 / 256;

    16 + (36 * ansi_r) + (6 * ansi_g) + ansi_b
}

fn draw_image(picture: &Path, width: u32, mode_8bit: bool) {
    // Load and resize the image
    let img = ImageReader::open(&picture).unwrap().decode().unwrap();
    let height = img.height() * width / img.width();
    let processed = if img.dimensions().0 != width {
        let resized = resize(&img, width, height, image::imageops::FilterType::Triangle);
        resized
    } else {
        img.as_rgba8().unwrap().clone()
    };

    // It will never be this value
    let mut last_rgb2 = 10000000;
    let mut last_rgb1 = 10000000;

    for y in (0..processed.dimensions().1).step_by(2) {
        for x in 0..processed.dimensions().0 {
            // Fetch the pixel rgb
            let pixel = processed.get_pixel(x, y);
            let r1 = pixel.0[0];
            let g1 = pixel.0[1];
            let b1 = pixel.0[2];
            let rgb1 = color8(r1 as usize, g1 as usize, b1 as usize);

            let string = if y < processed.dimensions().1 - 1 {
                let pixel = processed.get_pixel(x, y + 1);
                let r2 = pixel.0[0];
                let g2 = pixel.0[1];
                let b2 = pixel.0[2];
                let rgb2 = color8(r2 as usize, g2 as usize, b2 as usize);

                if mode_8bit {
                    if last_rgb2 == rgb2 && x != 0
                    {
                        if last_rgb1 == rgb1 {
                            format!("▄")
                        }
                        else {
                            last_rgb1 = rgb1;
                            format!("\x1b[48;5;{rgb1}m▄")
                        }
                    } else {
                        last_rgb2 = rgb2;
                        if last_rgb1 == rgb1 && x != 0 {
                            format!("\x1b[38;5;{rgb2}m▄")
                        }
                        else {
                            last_rgb1 = rgb1;
                            format!("\x1b[38;5;{rgb2};48;5;{rgb1}m▄")
                        }
                    }
                } else {
                    format!("\x1b[38;2;{r2};{g2};{b2};48;2;{r1};{g1};{b1}m▄")
                }
            } else {
                if mode_8bit {
                    if last_rgb1 == rgb1 && x != 0 {
                        format!("▀")
                    }
                    else {
                        last_rgb1 = rgb1;
                        format!("\x1b[38;5;{rgb1}m▀")
                    }
                } else {
                    format!("\x1b[49;38;2;{r1};{g1};{b1}m▀")
                }
            };
            print!("{}", string);
        }
        println!("\x1b[0m");
    }
}

fn main() {
    let mut args = env::args().skip(1);

    // Get the arguments from argv
    let path = args.next().expect("Needs 4 arguments.");

    let width = args.next()
            .expect("Needs 4 arguments.")
            .parse::<u32>()
            .expect("Invalid size.");

    let mode_8bit = args.next()
            .expect("Needs 3 arguments.");
    let mode_8bit = match mode_8bit.as_str() {
        "full" => false,
        "8bit" => true,
        _ => panic!("Color must be \"full or 8bit\"")
    };

    draw_image(
        Path::new(&path),
        width,
        mode_8bit
    );
}
