use tp_led_matrix::{Color, Image};

fn main() {
    // Tests for div and mul
    let color = Color { r: 200, g: 2, b: 1 };
    let divided_color = color / 2.0;
    println!("divided_color {:?}", divided_color);

    // Tests for gamma correction
    let color = Color { r: 200, g: 2, b: 1 };
    let gamma_corrected_color = color.gamma_correct();
    println!("gamma_corrected_color {:?}", gamma_corrected_color);

    // Tests for gradient
    let image = Image::gradient(Color::RED);
    println!("image.row(0) {:?}", image.row(0));

    // Tests for index and index_mut
    let mut image = Image::new_solid(Color::RED);
    image[(0, 0)].r = 255;
    println!("image[(0, 0)].r {:?}", image[(0, 0)].r);

    // Tests for AsRef and AsMut
    let mut image = Image::new_solid(Color::RED);
    let image_ref: &[u8; 192] = image.as_ref();
    println!("image_ref {:?}", image_ref);
    let image_mut: &mut [u8; 192] = image.as_mut();
    println!("image_mut {:?}", image_mut);
}