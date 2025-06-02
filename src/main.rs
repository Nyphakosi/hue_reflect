use std::{env, sync::{Arc, Mutex}, thread};

use image::{GenericImageView, ImageBuffer, Rgb, Rgba};

// rgb↔hsv conversion functions taken from https://gist.github.com/bmgxyz/a5b5b58e492cbca099b468eddd04cc97

struct Hsv([f32; 3]);

fn rgb_to_hsv(pixel: &Rgb<u8>) -> Hsv {
    let [r, g, b] = pixel.0;
    let big_m = *[r, g, b].iter().max().unwrap() as f32 / 255.;
    let little_m = *[r, g, b].iter().min().unwrap() as f32 / 255.;
    let c = big_m - little_m;
    let s = (c / big_m) * 100.;
    let (little_r, little_g, little_b) = (r as f32 / 255., g as f32 / 255., b as f32 / 255.);
    let (big_r, big_g, big_b) = (
        (big_m - little_r) / c,
        (big_m - little_g) / c,
        (big_m - little_b) / c,
    );
    let h_prime = match big_m {
        x if x == little_m => 0.,
        x if x == little_r => big_b - big_g,
        x if x == little_g => 2. + big_r - big_b,
        x if x == little_b => 4. + big_g - big_r,
        _ => unreachable!(),
    };
    let h = h_prime / 6. * 360.;
    let v = big_m * 100.;
    Hsv([h, s, v])
}

fn hsv_to_rgb(pixel: &Hsv) -> Rgb<u8> {
    let [hue, saturation, value] = [pixel.0[0], pixel.0[1] / 100., pixel.0[2] / 100.];
    let max = value;
    let c = saturation * value;
    let min = max - c;
    let h_prime = if hue >= 300. {
        (hue - 360.) / 60.
    } else {
        hue / 60.
    };
    let (r, g, b) = match h_prime {
        x if -1. <= x && x < 1. => {
            if h_prime < 0. {
                (max, min, min - h_prime * c)
            } else {
                (max, min + h_prime * c, min)
            }
        }
        x if 1. <= x && x < 3. => {
            if h_prime < 2. {
                (min - (h_prime - 2.) * c, max, min)
            } else {
                (min, max, min + (h_prime - 2.) * c)
            }
        }
        x if 3. <= x && x < 5. => {
            if h_prime < 4. {
                (min, min - (h_prime - 4.) * c, max)
            } else {
                (min + (h_prime - 4.) * c, min, max)
            }
        }
        _ => unreachable!(),
    };
    Rgb([(r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8])
}

fn hsv_reflect(pixel: &Hsv, reflect_angle: f32) -> Hsv {
    let [hue, saturation, value] = [pixel.0[0], pixel.0[1], pixel.0[2]];
    let mut angle = hue;

    angle = angle - reflect_angle;
    while angle < 0. {
        angle += 360.;
    }
    angle = 360. - angle;
    angle += reflect_angle;

    Hsv([angle, saturation, value])
}

fn main() {
    let timer = std::time::Instant::now();
    let core_count: u32 = num_cpus::get() as u32;
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: input a file path and reflect angle as command line arguments");
        return;
    }
    let file_path: &String = &args[1];
    let reflect_angle: f32 = (args[2].parse::<f32>().expect("Angle must be number")) % 180.;
    let img = Arc::new(image::open(file_path).expect("Failed to open image"));

    let (width, height) = img.dimensions();

    let new_img = Arc::new(Mutex::new(ImageBuffer::new(width, height)));

    let mut handles = vec![];

    let timer_elapsed = timer.elapsed();
    println!("Image loaded in {}ms", timer_elapsed.as_millis());

    println!("Processing...");
    // process main image
    for y in 0..height/core_count {
        for y_inner in 0..core_count {
            let img_clone = Arc::clone(&img);
            let new_img_clone = Arc::clone(&new_img);
            handles.push(thread::spawn(move || {
                for x in 0..width {
                    let pixel = img_clone.get_pixel(x, y*core_count+y_inner);
                    let pxl: Rgb<u8> = Rgb([pixel[0], pixel[1], pixel[2]]);
                    let hsv = rgb_to_hsv(&pxl);

                    let new_hsv = hsv_reflect(&hsv, reflect_angle);
                    let new_rgb = hsv_to_rgb(&new_hsv);
                    let new_pixel = Rgba([new_rgb[0], new_rgb[1], new_rgb[2], pixel[3]]);
                    new_img_clone.lock().unwrap().put_pixel(x, y*core_count+y_inner, new_pixel);
                }
            }));
        }
    }
    // process remainder of image
    for y in height/core_count*core_count..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let pxl: Rgb<u8> = Rgb([pixel[0], pixel[1], pixel[2]]);
            let hsv = rgb_to_hsv(&pxl);

            let new_hsv = hsv_reflect(&hsv, reflect_angle);
            let new_rgb = hsv_to_rgb(&new_hsv);
            let new_pixel = Rgba([new_rgb[0], new_rgb[1], new_rgb[2], pixel[3]]);
            new_img.lock().unwrap().put_pixel(x, y, new_pixel);
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let timer_elapsed = timer.elapsed();
    println!("Done in {}ms", timer_elapsed.as_millis());

    new_img.lock().unwrap().save("output.png").unwrap();
}

// fn inputf32() -> f32 {
//     loop {
//         let mut value = String::new();

//         io::stdin()
//             .read_line(&mut value)
//             .expect("Failed to read line");

//         match value.trim().parse() {
//             Ok(num) => return num,
//             Err(_) => continue,
//         };
//     }
// }

// fn inputstr() -> String {
//     loop {
//         let mut value = String::new();

//         io::stdin()
//             .read_line(&mut value)
//             .expect("Failed to read line");

//         match value.trim().parse() {
//             Ok(num) => return num,
//             Err(_) => continue,
//         };
//     }
// }