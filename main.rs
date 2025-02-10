mod image_procs_n_raytrace;
use image_procs_n_raytrace::*;

fn main() {
    //let img: ImageBuffer<Rgb<u8>, Vec<u8>> = raytrace(vec![], [256, 256], Xyz::new([0.0, 0.0, 0.0]), 0, Rgb([0, 0, 0]));

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = open_img("image.png").unwrap();
    let mut rimg: &mut ImageBuffer<Rgb<u8>, Vec<u8>> = &mut img;

    let pal: Vec<[i32; 3]> = vec![
        [0, 0, 0],
        [40, 30, 40],
        [75, 50, 75],
        [100, 66, 100],
        [200, 132, 200],
    ];
    //let pal: Vec<[i32; 3]> = vec![[0, 0, 0], [255, 255, 255]];
    img = rec_to_quad(rimg);
    rimg = &mut img;
    img = errprop_dithering(rimg, pal, 4);
    rimg = &mut img;
    add(rimg, Rgb([40, 40, 40]));
    colorize(rimg, Rgb([255, 175, 255]));
    img = upscale(rimg, 2);
    let _ = save_img(img, "res.jpg");
}
