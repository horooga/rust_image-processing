mod image_procs_n_raytrace;
use image_procs_n_raytrace::*;

fn main() {
    //let img: ImageBuffer<Rgb<u8>, Vec<u8>> = raytrace(vec![], [256, 256], Xyz::new([0.0, 0.0, 0.0]), 0, Rgb([0, 0, 0]));

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = open_img("image.jpg").unwrap();
    img = downscale(img, 8);
    let mut rimg: &mut ImageBuffer<Rgb<u8>, Vec<u8>> = &mut img;

    to_n_val_channels(rimg, 6);
    let colors: Vec<[i32; 3]> = to_colors(rimg);
    //print!("{:?}", colors);

    img = ord_bayer_dithering(img, colors, BAYER_8X8, 2, 1.0, 0.0);

    rimg = &mut img;
    bright(rimg, 1.3);
    to_mc_pic(rimg, 3, Rgb([60, 20, 20]));

    let _ = save_img(img, "res.jpg");
}
