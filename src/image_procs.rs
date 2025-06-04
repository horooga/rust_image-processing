use crate::types_n_convs::*;
pub use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
pub use std::fs::File;
pub use std::io::Read;
use std::io::{Error, Write};
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

fn get_n_index(v: String) -> usize {
    let mut c: i32 = 0;
    for i in 0..v.len() {
        if v.chars().nth(i).unwrap() == '/' {
            c += 1;
        } else if c == 2 {
            return v[i..].parse().unwrap();
        }
    }
    return 0;
}

fn get_v_indexes(line: Vec<String>) -> Vec<usize> {
    let mut output: Vec<usize> = vec![];
    for i in 1..line.len() {
        let v: String = line[i].clone();
        for j in 0..v.len() {
            if v.chars().nth(j).unwrap() == '/' {
                output.push(v[..j].parse().unwrap());
                break;
            }
        }
    }
    return output;
}

fn rgb_dist(one: [i32; 3], two: [i32; 3]) -> u16 {
    ((i32_sub(one[0], two[0]).pow(2)
        + i32_sub(one[1], two[1]).pow(2)
        + i32_sub(one[2], two[2]).pow(2)) as f32)
        .sqrt() as u16
}

fn closest_color(colors_from: Vec<[i32; 3]>, color_to: [i32; 3]) -> Rgb<u8> {
    let mut mindist: u16 = 65535;
    let mut dist: u16;
    let mut color_index: usize = 0;
    let mut color: [i32; 3];
    for ci in 0..colors_from.len() {
        color = colors_from[ci];
        dist = rgb_dist(color, color_to);
        if dist < mindist {
            mindist = dist;
            color_index = ci;
        }
    }
    let res: [i32; 3] = colors_from[color_index];
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
    pal: Vec<[i32; 3]>,
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

pub fn obj_import(name: &str, pos: [f32; 3], rfty: f32) -> std::io::Result<Vec<Object>> {
    let mut output: Vec<Object> = vec![];
    let mut f = File::open(name)?;
    let mut data: Vec<u8> = vec![];
    f.read_to_end(&mut data)?;
    let mut curr: Vec<u8> = vec![];
    let mut lines: Vec<String> = vec![];
    let mut ns: Vec<[f32; 3]> = vec![];
    let mut vs: Vec<[f32; 3]> = vec![];
    let mut curr_color: Rgb<u8> = Rgb([0, 0, 0]);
    let mut light: bool = false;
    for i in data.into_iter() {
        if i == 10 {
            lines.push(String::from_utf8(curr).unwrap());
            curr = vec![];
        } else if i != 13 {
            curr.push(i)
        }
    }
    lines.push(String::from_utf8(curr).unwrap());
    for l in lines.clone().into_iter() {
        let line: Vec<String> = l.split(" ").map(|s| s.to_string()).collect();
        match line[0].as_str() {
            "color" => {
                curr_color = Rgb([
                    line[1].parse::<u8>().unwrap(),
                    line[2].parse::<u8>().unwrap(),
                    line[3].parse::<u8>().unwrap(),
                ]);
                light = false;
            }
            "light" => {
                light = true;
            }
            "v" => {
                vs.push([
                    -line[3].parse::<f32>().unwrap() + pos[0],
                    line[1].parse::<f32>().unwrap() + pos[1],
                    line[2].parse::<f32>().unwrap() + pos[2],
                ]);
            }
            "vn" => ns.push([
                -line[3].parse::<f32>().unwrap(),
                line[1].parse::<f32>().unwrap(),
                line[2].parse::<f32>().unwrap(),
            ]),
            "f" => {
                if line.len() == 4 {
                    let cvs: Vec<usize> = get_v_indexes(line.clone());
                    output.push(Object::new_triangle(
                        Xyz::to_xyz(ns[get_n_index(line[1].clone()) - 1]),
                        curr_color,
                        Xyz::to_xyz(vs[cvs[0] - 1]),
                        Xyz::to_xyz(vs[cvs[1] - 1]),
                        Xyz::to_xyz(vs[cvs[2] - 1]),
                        rfty * if light { 50.0 } else { 1.0 },
                        light,
                    ))
                } else if line.len() == 5 {
                    let cvs: Vec<usize> = get_v_indexes(line.clone());
                    output.push(Object::new_triangle(
                        Xyz::to_xyz(ns[get_n_index(line[1].clone()) - 1]),
                        curr_color,
                        Xyz::to_xyz(vs[cvs[0] - 1]),
                        Xyz::to_xyz(vs[cvs[1] - 1]),
                        Xyz::to_xyz(vs[cvs[2] - 1]),
                        rfty * if light { 50.0 } else { 1.0 },
                        light,
                    ));
                    output.push(Object::new_triangle(
                        Xyz::to_xyz(ns[get_n_index(line[1].clone()) - 1]),
                        curr_color,
                        Xyz::to_xyz(vs[cvs[0] - 1]),
                        Xyz::to_xyz(vs[cvs[2] - 1]),
                        Xyz::to_xyz(vs[cvs[3] - 1]),
                        rfty * if light { 50.0 } else { 1.0 },
                        light,
                    ));
                }
            }
            _ => (),
        }
    }
    return Ok(output);
}

pub fn raytrace(
    objects: Vec<Object>,
    res: [u32; 2],
    ro: Xyz,
    rs: u8,
    bgd_col: Rgb<u8>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = RgbImage::new(res[1], res[0]);
    let mut lights: Vec<Object> = vec![];
    for oi in 0..objects.len() {
        if objects[oi].light && lights.len() < 100 {
            lights.push(objects[oi]);
        }
    }
    let r_y: f32 = res[0] as f32;
    let r_x: f32 = res[1] as f32;
    let mut intersect: Intersection = Intersection::null();
    let mut pre_intersect: Intersection = Intersection::null();
    let mut rfty: f32;
    let mut buff_intersect: Intersection;
    for y in 0..res[0] {
        println!("{}", (y as f32 / res[0] as f32 * 100.0) as u32);
        let y_d: f32 = -(y as f32 / r_y * 2.0 - 1.0);
        for x in 0..res[1] {
            let x_d: f32 = x as f32 / r_x * 2.0 - 1.0;
            let mut color: Rgb<u8> = Rgb([0, 0, 0]);
            let rd: Xyz = Xyz::new([1.0, x_d, y_d]).norm();
            let mut lro: Xyz = ro;
            let mut lrd: Xyz = rd;
            let mut any: bool = false;
            rfty = 1.0;
            let mut refl_obj: Object = Object::null();
            for _ in 0..rs {
                intersect.t = 99999.0;
                pre_intersect.t = 99999.0;
                for i in objects.clone() {
                    pre_intersect = i.check(lro, lrd);
                    if pre_intersect.t != -1.0
                        && pre_intersect.t < intersect.t
                        && (pre_intersect.hit_obj != refl_obj)
                    {
                        intersect = pre_intersect;
                    }
                }
                if intersect.t != 99999.0 {
                    refl_obj = intersect.hit_obj;
                    let mut shadow: f32 = 1.0;
                    any = true;
                    if !intersect.hit_obj.light {
                        lro = intersect.hit;
                        lrd = lrd.reflect(intersect.n);
                        let mut light_c: Xyz = Xyz::new([1.0, 1.0, 1.0]);
                        for li in 0..lights.len() {
                            let light: Object = lights[li];
                            light_c = light_c
                                .mul(
                                    1.0 - (intersect.n.dot((lro.xyz_sub(light.b)).norm()) + 1.0)
                                        / 2.0,
                                )
                                .xyz_mul(Xyz::new([
                                    light.col[0] as f32 / 255.0,
                                    light.col[1] as f32 / 255.0,
                                    light.col[2] as f32 / 255.0,
                                ]));
                            let mut light_intersects: Vec<Intersection> = vec![];
                            for oi in 0..objects.len() {
                                let object: Object = objects[oi];
                                if !object.light && object != intersect.hit_obj {
                                    buff_intersect =
                                        object.check(lro, light.pos.xyz_sub(lro).norm());
                                    if buff_intersect.t != -1.0
                                        && intersect.t < light.pos.xyz_sub(lro).length()
                                    {
                                        light_intersects.push(buff_intersect);
                                    }
                                }
                            }
                            if light_intersects.len() != 0 {
                                shadow = 0.5;
                            }
                        }
                        color = rgb_mul(
                            rgb_add(
                                color,
                                rgb_xyz_mul(rgb_mul(intersect.hit_obj.col, rfty), light_c),
                            ),
                            shadow,
                        );
                        rfty = intersect.hit_obj.rfty;
                    } else {
                        color = rgb_add(color, intersect.hit_obj.col);
                        break;
                    }
                } else {
                    break;
                }
            }
            if !any {
                img.put_pixel(x, y, bgd_col)
            } else {
                img.put_pixel(x, y, color);
            }
        }
    }
    return img;
}

pub fn open_img(path: &str) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageReader::open(path)?.decode()?.into_rgb8();
    return Ok(img);
}

pub fn save_img(img: ImageBuffer<Rgb<u8>, Vec<u8>>, path: &str) -> Result<(), image::ImageError> {
    let _ = DynamicImage::ImageRgb8(img).save(path);
    return Ok(());
}
