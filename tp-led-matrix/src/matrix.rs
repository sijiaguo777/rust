use embassy_stm32::gpio::*;
use embassy_stm32::peripherals::*;
use embassy_time::Ticker;
use embassy_time::{Duration, Timer};
use crate::{Color, Image};
pub struct Matrix<'a> {
    sb: Output<'a>,
    lat: Output<'a>,
    rst: Output<'a>,
    sck: Output<'a>,
    sda: Output<'a>,
    pub rows: [Output<'a>; 8],
}

impl Matrix<'_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        pa2: PA2,
        pa3: PA3,
        pa4: PA4,
        pa5: PA5,
        pa6: PA6,
        pa7: PA7,
        pa15: PA15, // <Alternate<PushPull, 0>>,
        pb0: PB0,
        pb1: PB1,
        pb2: PB2,
        pc3: PC3,
        pc4: PC4,
        pc5: PC5,
    ) -> Self {
        // Configure the pins, with the correct speed and their initial state
        let sb = Output::new(pc5, Level::High, Speed::VeryHigh);
        let lat = Output::new(pc4, Level::High, Speed::VeryHigh);
        let rst = Output::new(pc3, Level::Low, Speed::VeryHigh);
        let sck = Output::new(pb1, Level::Low, Speed::VeryHigh);
        let sda = Output::new(pa4, Level::Low, Speed::VeryHigh);
        let rows = [
            Output::new(pb2, Level::Low, Speed::VeryHigh),
            Output::new(pa15, Level::Low, Speed::VeryHigh),
            Output::new(pa2, Level::Low, Speed::VeryHigh),
            Output::new(pa7, Level::Low, Speed::VeryHigh),
            Output::new(pa6, Level::Low, Speed::VeryHigh),
            Output::new(pa5, Level::Low, Speed::VeryHigh),
            Output::new(pb0, Level::Low, Speed::VeryHigh),
            Output::new(pa3, Level::Low, Speed::VeryHigh),
        ];

        let mut instance = Self {
            sb,
            lat,
            rst,
            sck,
            sda,
            rows,
        };

        Timer::after(Duration::from_millis(100)).await;
        
        instance.init_bank0();
        instance.rst.set_high();
        instance
    }

    /// Make a brief high pulse of the SCK pin
    fn pulse_sck(&mut self) {
        self.sck.set_high();
        self.sck.set_low();
    }


    /// Make a brief low pulse of the LAT pin
    fn pulse_lat(&mut self) {
        self.lat.set_low();
        self.lat.set_high();
    }


    fn send_byte(&mut self, pixel: u8) {
        for i in (0..8).rev() {
            let bit = (pixel >> i) & 1;
            if bit == 1 {
                self.sda.set_high();
            } else {
                self.sda.set_low();
            }
            self.pulse_sck();
        }
    }


    pub fn send_row(&mut self, row: usize, pixels: &[Color]) {
        // Send the new row
        for pixel in pixels.iter().rev() {
            let pixel = pixel.gamma_correct();
            self.send_byte(pixel.b);
            self.send_byte(pixel.g);
            self.send_byte(pixel.r);
        }

        for r in &mut self.rows {
            r.set_low();
        }

        // Pulse LAT
        self.pulse_lat();

        // Activate the new row
        self.rows[row].set_high();
    }

    
    pub fn init_bank0(&mut self) {
        self.sb.set_low();
        for _ in 0..18 {
            self.send_byte(0xFF);
        }
        self.pulse_lat();
        self.sb.set_high();
    }

    /// Display a full image, row by row, as fast as possible.
    pub async fn display_image(&mut self, image: &Image, ticker: &mut Ticker) {
        // Do not forget that image.row(n) gives access to the content of row n,
        // and that self.send_row() uses the same format.


        for row in 0..8 {
            self.send_row(row, &image.row(row));
            ticker.next().await;
        }
        for r in &mut self.rows {
            r.set_low();
        }
    }
}

