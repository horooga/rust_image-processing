mod image_procs_n_raytrace;
use image_procs_n_raytrace::*;

fn main() {
    //let img: ImageBuffer<Rgb<u8>, Vec<u8>> = raytrace(vec![], [256, 256], Xyz::new([0.0, 0.0, 0.0]), 0, Rgb([0, 0, 0]));

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = open_img("image.png").unwrap();
    let mut rimg: &mut ImageBuffer<Rgb<u8>, Vec<u8>> = &mut img;

    let pal: Vec<[i32; 3]> = vec![
        [20, 20, 20],
        [55, 34, 55],
        [75, 50, 75],
        [100, 66, 100],
        //[180, 120, 180],
        [255, 180, 255],
    ];
    img = rec_to_quad(rimg);
    rimg = &mut img;
    edit_color(rimg, vec![Rgb([0, 0, 0]), Rgb([80, 30, 80])]);
    //img = ord_bayer_dithering(img, pal, BAYER_8X8, 4, 1.7, 70.0);
    img = twod_errprop_dithering(img, pal, 4, 1.8, 0.7);
    rimg = &mut img;
    colorize(rimg, Rgb([255, 175, 255]));
    bright(rimg, 1.5);
    img = upscale(rimg, 2);
    let _ = save_img(img, "res.jpg");
}
