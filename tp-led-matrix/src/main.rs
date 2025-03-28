#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_stm32::usart::{self, Uart};
use embassy_time::Ticker;
use embassy_time::{Duration, Timer};

use embassy_stm32::peripherals::USART1;
use embassy_stm32::peripherals::*;
use embassy_stm32::{bind_interrupts, gpio::*};

use defmt_rtt as _;
use embassy_stm32::rcc::*;
use embassy_stm32::Config;
use panic_probe as _;
use tp_led_matrix::matrix;
use tp_led_matrix::matrix::Matrix;
use tp_led_matrix::Color;
use tp_led_matrix::Image;
extern crate embassy_executor;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;

static IMAGE: Mutex<ThreadModeRawMutex, Image> = Mutex::new(Image::new_solid(Color::BLACK));

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
    let matrix = matrix::Matrix::new(
        p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,
    )
    .await;

    spawner
        .spawn(serial_receiver(
            p.USART1, p.PB7, p.PB6, p.DMA1_CH4, p.DMA1_CH5,
        ))
        .unwrap();
    spawner.spawn(display(matrix)).unwrap();
    // spawner.spawn(image_change()).unwrap();
}

#[embassy_executor::task]
async fn blinker(pb14: PB14) {
    let mut led = Output::new(pb14, Level::Low, Speed::Low);
    loop {
        for _ in 0..3 {
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
    matrix.init_bank0();
    loop {
        for r in 0..8 {
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

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
});

#[embassy_executor::task]
async fn serial_receiver(
    usart1: USART1,
    tx_pin: PB7,
    rx_pin: PB6,
    dma_tx: DMA1_CH4,
    dma_rx: DMA1_CH5,
) {
    let mut config = embassy_stm32::usart::Config::default();
    config.baudrate = 38400;
    let mut uart = Uart::new(usart1, tx_pin, rx_pin, Irqs, dma_tx, dma_rx, config)
        .expect("Failed to initialize UART");

    let mut buffer: [u8; 192] = [0; 192];
    let mut n;

    loop {
        loop {
            let mut byte_buf = [0u8; 1];
            uart.read(&mut byte_buf).await.expect("failed to read byte");
            if byte_buf[0] == 0xff {
                n = 0;
                break;
            }
        }

        while n < 192 {
            let mut byte_buf = [0u8; 1];
            uart.read(&mut byte_buf).await.expect("failed to read byte");
            let byte = byte_buf[0];
            buffer[n] = byte;
            n += 1;

            if let Some(k) = buffer[..n].iter().rposition(|&x| x == 0xff) {
                if k != 0 {
                    buffer.rotate_right(k);
                    n = k;
                    continue;
                }
            }

            if n == 192 {
                {
                    let mut image = IMAGE.lock().await;
                    let new_image = Image::from_buffer(&buffer);
                    *image = new_image;
                }
                break;
            }
        }
    }
}

#[embassy_executor::task]
async fn image_change() {
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
