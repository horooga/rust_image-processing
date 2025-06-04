use crate::types_n_convs::*;
pub use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
pub use std::fs::File;
pub use std::io::Read;
use std::io::{Error, Write};
use std::path::Path;
pub use std::str;
pub use std::{cmp, vec};
pub use Vec;

pub const BAYER_8X8: [[u8; 8]; 8] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

fn rgb_dist(one: [u8; 3], two: [u8; 3]) -> u32 {
    ((i32_sub(one[0], two[0]).pow(2)
        + i32_sub(one[1], two[1]).pow(2)
        + i32_sub(one[2], two[2]).pow(2)) as f32)
        .sqrt() as u32
}

fn closest_color(colors_from: Vec<[u8; 3]>, color_to: [u8; 3]) -> Rgb<u8> {
    let mut mindist: u32 = 65535;
    let mut dist: u32;
    let mut color_index: usize = 0;
    let mut color: [u8; 3];
    for ci in 0..colors_from.len() {
        color = colors_from[ci];
        dist = rgb_dist(color, color_to);
        if dist < mindist {
            mindist = dist;
            color_index = ci;
        }
    }
    let res: [u8; 3] = colors_from[color_index];
    Rgb([
        if res[0] < 0 {
            0
        } else if res[0] > 255 {
            255
        } else {
            res[0] as u8
        },
        if res[1] < 0 {
            0
        } else if res[1] > 255 {
            255
        } else {
            res[1] as u8
        },
        if res[2] < 0 {
            0
        } else if res[2] > 255 {
            255
        } else {
            res[2] as u8
        },
    ])
}

fn closest_index(max_index: usize, val_to: f32) -> usize {
    let mut mindist: f32 = 65535.0;
    let mut dist: f32;
    let mut index: usize = 0;
    for i in 0..max_index {
        dist = (val_to - i as f32).abs();
        if dist < mindist {
            mindist = dist;
            index = i;
        }
    }
    return index;
}

fn measure_equal(one: Rgb<u8>, two: Rgb<u8>, measure: i32) -> bool {
    for i in 0..3 {
        if (two[i as usize] as i32 - one[i as usize] as i32).abs() > measure {
            return false;
        }
    }
    return true;
}

pub fn ord_bayer_dithering(
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pal: Vec<[u8; 3]>,
    mat: [[u8; 8]; 8],
    pixel_size: u32,
    d: f32,
    m: f32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (x, y) = img.dimensions();
    let mut nimg: ImageBuffer<Rgb<u8>, Vec<u8>> =
        RgbImage::new(x / pixel_size + 1, y / pixel_size + 1);
    let mut ic = 0;
    for i in (0..y).step_by(pixel_size as usize) {
        let mut jc = 0;
        for j in (0..x).step_by(pixel_size as usize) {
            let val: f32 =
                mat[(ic as f32 % 8.0) as usize][(jc as f32 % 8.0) as usize] as f32 / 64.0;
            let color: Rgb<u8> = closest_color(
                pal.clone(),
                rgb_to_arr(rgb_add(
                    rgb_div(*img.get_pixel(j, i), d),
                    u8_to_rgb((m * val) as u8),
                )),
            );
            nimg.put_pixel(jc, ic, color);
            jc += 1;
        }
        ic += 1;
    }
    nimg
}

pub fn twod_errprop_dithering(
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pal: Vec<[i32; 3]>,
    pixel_size: u32,
    d: f32,
    m: f32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (x, y) = img.dimensions();
    let mut nimg: ImageBuffer<Rgb<u8>, Vec<u8>> =
        RgbImage::new(x / pixel_size + 1, y / pixel_size + 1);
    let mut ic = 0;
    let mut jc;
    let mut err_line: Vec<[i32; 3]> = vec![[0; 3]; (x / pixel_size) as usize];
    let mut curr_pixel: Rgb<u8>;
    let mut upper_pixel: Rgb<u8>;
    let mut color: Rgb<u8>;
    for i in (1..y).step_by(pixel_size as usize) {
        let mut err = [0; 3];
        jc = 0;
        for j in (0..x).step_by(pixel_size as usize) {
            curr_pixel = *img.get_pixel(j, i);
            upper_pixel = *img.get_pixel(j, i - 1);
            color = closest_color(
                pal.clone(),
                [
                    (curr_pixel[0] as f32 / d) as i32
                        + ((err[0] + err_line[0][0]) as f32 * m) as i32,
                    (curr_pixel[1] as f32 / d) as i32
                        + ((err[1] + err_line[0][1]) as f32 * m) as i32,
                    (curr_pixel[2] as f32 / d) as i32
                        + ((err[2] + err_line[0][2]) as f32 * m) as i32,
                ],
            );
            nimg.put_pixel(jc, ic, color);
            err = [
                curr_pixel[0] as i32 - color[0] as i32,
                curr_pixel[1] as i32 - color[1] as i32,
                curr_pixel[2] as i32 - color[2] as i32,
            ];
            err_line[jc as usize] = [
                upper_pixel[0] as i32 - color[0] as i32,
                upper_pixel[1] as i32 - color[1] as i32,
                upper_pixel[2] as i32 - color[2] as i32,
            ];
            jc += 1;
        }
        ic += 1;
    }
    nimg
}

pub fn ascii_ord_bayer_dithering(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    pal: Vec<char>,
    mat: [[u8; 8]; 8],
    k: f32,
    ssr: f32,
    r: f32,
) -> Result<(), Error> {
    let mut output = File::create("ascii.txt")?;
    let (x, y) = img.dimensions();
    let pal_size: usize = pal.len();
    let kh: f32 = k * ssr;
    for i in 0..y {
        if (i as f32 % kh) as i32 == 0 {
            for j in 0..x {
                if (j as f32 % k) as i32 == 0 {
                    let pixel: Rgb<u8> = *img.get_pixel(j, i);
                    let brightness: f32 =
                        (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 255.0 / 3.0;
                    let val: f32 = mat[((i / k as u32) as f32 % 8.0) as usize]
                        [((j / k as u32) as f32 % 8.0) as usize]
                        as f32
                        / 64.0;
                    let c_index = closest_index(pal_size, brightness * pal_size as f32 + r * val);
                    write!(output, "{}", pal[c_index])?;
                }
            }
        }
    }
    Ok(())
}

pub fn to_colors(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<[i32; 3]> {
    let (x, y) = img.dimensions();
    let mut colors: Vec<[i32; 3]> = vec![];
    let mut pixel: [u8; 3];
    let mut color: [i32; 3];
    for i in 0..y {
        for j in 0..x {
            pixel = img.get_pixel(j, i).0;
            color = [pixel[0] as i32, pixel[1] as i32, pixel[2] as i32];
            if !(colors.contains(&color)) {
                colors.push(color);
            }
        }
    }
    colors
}

pub fn to_n_val_channels(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, mut n: u8) {
    let (x, y) = img.dimensions();
    let mut pixel_val: [u8; 3];
    n = 255 / n;
    for i in 0..y {
        for j in 0..x {
            pixel_val = img.get_pixel(j, i).0;
            img.put_pixel(
                j,
                i,
                Rgb([
                    (pixel_val[0] as f32 / n as f32) as u8 * n,
                    (pixel_val[1] as f32 / n as f32) as u8 * n,
                    (pixel_val[2] as f32 / n as f32) as u8 * n,
                ]),
            )
        }
    }
}

pub fn edit_color(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, colors: Vec<Rgb<u8>>, measure: i32) {
    let (x, y) = img.dimensions();
    for i in 0..y {
        for j in 0..x {
            for k in (0..colors.len()).step_by(2) {
                if measure_equal(colors[k], *img.get_pixel(j, i), measure) {
                    img.put_pixel(j, i, colors[k + 1]);
                }
            }
        }
    }
}

pub fn upscale(img: ImageBuffer<Rgb<u8>, Vec<u8>>, k: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (x, y) = img.dimensions();
    let mut nimg: ImageBuffer<Rgb<u8>, Vec<u8>> = RgbImage::new(x * k, y * k);
    for i in 0..(y * k) {
        for j in 0..(x * k) {
            nimg.put_pixel(j, i, *img.get_pixel(j / k, i / k));
        }
    }
    nimg
}

pub fn downscale(img: ImageBuffer<Rgb<u8>, Vec<u8>>, k: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (x, y) = img.dimensions();
    let mut nimg: ImageBuffer<Rgb<u8>, Vec<u8>> = RgbImage::new(x / k + 1, y / k + 1);
    let mut ic: u32 = 0;
    for i in (0..y).step_by(k as usize) {
        let mut jc: u32 = 0;
        for j in (0..x).step_by(k as usize) {
            nimg.put_pixel(jc, ic, *img.get_pixel(j, i));
            jc += 1;
        }
        ic += 1;
    }
    nimg
}

pub fn pinkize(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    let (x, y) = img.dimensions();
    for i in 0..y {
        for j in 0..x {
            let pixel: Rgb<u8> = *img.get_pixel(j, i);
            let c_max: u8 = cmp::max(pixel[2], cmp::max(pixel[0], pixel[1]));
            let c_min: u8 = cmp::min(pixel[2], cmp::min(pixel[0], pixel[1]));
            img.put_pixel(j, i, Rgb([c_max, c_min, (c_max as f32 * 0.8) as u8]));
        }
    }
}

pub fn bright(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, c: f32) {
    let (x, y) = img.dimensions();
    for i in 0..y {
        for j in 0..x {
            let pixel: Rgb<u8> = *img.get_pixel(j, i);
            img.put_pixel(
                j,
                i,
                Rgb([
                    u8_mul(pixel[0], c),
                    u8_mul(pixel[1], c),
                    u8_mul(pixel[2], c),
                ]),
            );
        }
    }
}

pub fn colorize(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, color: Rgb<u8>) {
    let (x, y) = img.dimensions();
    for i in 0..y {
        for j in 0..x {
            let pixel: Rgb<u8> = *img.get_pixel(j, i);
            let brightness: f32 = (pixel[0] as f32 / 255.0
                + pixel[1] as f32 / 255.0
                + pixel[2] as f32 / 255.0) as f32;
            img.put_pixel(j, i, rgb_mul(color, brightness));
        }
    }
}

pub fn add(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, color: Rgb<u8>) {
    let (x, y) = img.dimensions();
    for i in 0..y {
        for j in 0..x {
            img.put_pixel(j, i, rgb_add(*img.get_pixel(j, i), color));
        }
    }
}

pub fn to_mc_pic(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, width: u32, frame_color: Rgb<u8>) {
    let (x, y) = img.dimensions();

    for i in 0..(y - 1) {
        for w in 0..width {
            img.put_pixel(0 + w, i, frame_color);
            img.put_pixel(x - 1 - w, i, frame_color);
        }
    }
    for j in 0..(x - 1) {
        for w in 0..width {
            img.put_pixel(j, 0 + w, frame_color);
            img.put_pixel(j, y - 1 - w, frame_color);
        }
    }
}

pub fn open_img(path: &str) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageReader::open(path)?.decode()?.into_rgb8();
    return Ok(img);
}

pub fn save_img(img: ImageBuffer<Rgb<u8>, Vec<u8>>, path: &str) -> Result<(), image::ImageError> {
    let _ = DynamicImage::ImageRgb8(img).save(path);
    return Ok(());
}
