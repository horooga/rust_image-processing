mod image_procs_n_raytrace;
use image_procs_n_raytrace::*;

fn main() {
    //let img: ImageBuffer<Rgb<u8>, Vec<u8>> = raytrace(vec![], [256, 256], Xyz::new([0.0, 0.0, 0.0]), 0, Rgb([0, 0, 0]));

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = open_img("image.png").unwrap();
    let mut rimg: &mut ImageBuffer<Rgb<u8>, Vec<u8>> = &mut img;

    let pal: Vec<[i32; 3]> = vec![
        [0, 0, 0],
        [75, 50, 75],
        [100, 66, 100],
        [255, 180, 255],
        [255, 255, 255],
    ];
    //let pal: Vec<[i32; 3]> = vec![[0, 0, 0], [255, 255, 255]];
    img = rec_to_quad(rimg);
    rimg = &mut img;
    img = twod_errprop_dithering(rimg, pal, 4);
    rimg = &mut img;
    colorize(rimg, Rgb([255, 175, 255]));
    edit_color(rimg, vec![Rgb([0, 0, 0]), Rgb([80, 30, 80])]);
    img = upscale(rimg, 2);
    let _ = save_img(img, "res.jpg");
}
