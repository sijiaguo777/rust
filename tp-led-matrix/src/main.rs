#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use embassy_stm32 as _;   
use embassy_time::Ticker;
use embassy_time::{Timer, Duration};
use embassy_stm32::gpio::*;
use embassy_stm32::peripherals::*;
// Just to link it in the executable (it provides the vector table)
use panic_probe as _;
use defmt_rtt as _;
use embassy_stm32::rcc::*;
use embassy_stm32::Config;
use tp_led_matrix::matrix::Matrix;
use tp_led_matrix::Image;
use tp_led_matrix::Color;
use tp_led_matrix::matrix;
extern crate embassy_executor;
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

static IMAGE: Mutex<ThreadModeRawMutex, Image> = Mutex::new(Image::new_solid(Color::GREEN));

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

    spawner.spawn(blinker(p.PB14)).unwrap();
    let matrix = matrix::Matrix::new(p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,).await;
    
    spawner.spawn(display(matrix)).unwrap();
    let mut current_color = 0;
    loop {
        let new_image = match current_color {
            0 => Image::gradient(Color::BLUE),
            1 => Image::gradient(Color::GREEN),
            2 => Image::gradient(Color::RED),
            _ => unreachable!(),
        };
        current_color = (current_color + 1) % 3;
        {
        let mut image = IMAGE.lock().await;
        *image = new_image;
        }
        Timer::after(Duration::from_secs(1)).await;
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


#[embassy_executor::task]
async fn display(mut matrix: Matrix<'static>) {
    let mut ticker = Ticker::every(Duration::from_hz(640));
    // let mut local_image = Image::new_solid(Color::BLACK); // 或其他默认值
    matrix.init_bank0();
    loop{
        
        for r in 0..8{
            let mut buffer: [Color; 8] = [Color::default(); 8];
            {
                let image = IMAGE.lock().await;
                buffer.copy_from_slice(&image.row(r));
            }
            matrix.send_row(r, &buffer);
            ticker.next().await;
        }
        for r in &mut matrix.rows {
            r.set_low();
        }
    }
}