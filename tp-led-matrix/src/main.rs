#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32 as _;   // Just to link it in the executable (it provides the vector table)
use panic_probe as _;
use defmt_rtt as _;
use embassy_stm32::rcc::*;
use embassy_stm32::Config;
use tp_led_matrix::Image;
use tp_led_matrix::Color;
use tp_led_matrix::matrix;

#[entry]
fn main() -> ! {
    // defmt::info!("defmt correctly initialized");

    // Setup the clocks at 80MHz using HSI (by default since HSE/MSI
    // are not configured): HSI(16MHz)×10/2=80MHz. The flash wait
    // states will be configured accordingly.

    let mut config = Config::default();
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2), // 16 * 10 / 2 = 80MHz
    });
    config.rcc.sys = Sysclk::PLL1_R;
    let p = embassy_stm32::init(config);

    // In your main program, build an image made of a gradient of blue and display it in loop on the matrix. Since it is necessary for the display to go fast, do not forget to run your program in release mode, as we have been doing for a while now. Don't forget that Image values have a .row() method which can be handy here.

    let mut matrix = matrix::Matrix::new(p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,);

    let blue = Color::BLUE;
    let image= Image::gradient(blue);
    // for row in 0..8 {
    //     let row_pixels = image.row(row);
    //     defmt::info!("Row {}:", row);
    //     for (col, pixel) in row_pixels.iter().enumerate() {
    //         defmt::info!(
    //             "  Col {}: r = {=u8}, g = {=u8}, b = {=u8}",
    //             col,
    //             pixel.r,
    //             pixel.g,
    //             pixel.b
    //         );
    //     }
    // }
    
    loop {
        matrix.display_image(&image);
    }
    // panic!("Everything configured");


}