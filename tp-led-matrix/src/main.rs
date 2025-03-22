#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use embassy_stm32::spi::Polarity;
use embassy_stm32 as _;   
use embassy_time::{Timer, Duration};
use embassy_stm32::gpio::*;
use embassy_stm32::peripherals::*;
// Just to link it in the executable (it provides the vector table)
use panic_probe as _;
use defmt_rtt as _;
use embassy_stm32::rcc::*;
use embassy_stm32::Config;
use tp_led_matrix::Image;
use tp_led_matrix::Color;
use tp_led_matrix::matrix;
extern crate embassy_executor;
use embassy_executor::Spawner;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
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
    spawner.spawn(blinker(p.PB14)).unwrap();
    let mut matrix = matrix::Matrix::new(p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,).await;

    let blue = Color::BLUE;
    let image= Image::gradient(blue);
    
    loop {
        matrix.display_image(&image);
    }

}

#[embassy_executor::task]
async fn blinker(pb14: PB14){
    let mut led = Output::new(pb14, Level::Low, Speed::Low);
    loop {
        for _ in 0..3{
            led.set_high();
            Timer::after(Duration::from_millis(100)).await;
            led.set_low();
            Timer::after(Duration::from_millis(100)).await;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}
