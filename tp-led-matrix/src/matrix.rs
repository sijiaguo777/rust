use embassy_stm32::gpio::*;
use embassy_stm32::peripherals::*;
use embedded_hal::delay::DelayNs as _;
use embassy_time::Delay;
use crate::{Color, Image};
pub struct Matrix<'a> {
    sb: Output<'a>,
    lat: Output<'a>,
    rst: Output<'a>,
    sck: Output<'a>,
    sda: Output<'a>,
    rows: [Output<'a>; 8],
}

impl Matrix<'_> {
    /// Create a new matrix from the control registers and the individual
    /// unconfigured pins. SB and LAT will be set high by default, while
    /// other pins will be set low. After 100ms, RST will be set high, and
    /// the bank 0 will be initialized by calling `init_bank0()` on the
    /// newly constructed structure.
    /// The pins will be set to very high speed mode.
    #[allow(clippy::too_many_arguments)]   // Necessary to avoid a clippy warning
    pub fn new(
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
        let sb = Output::new(pa2, Level::High, Speed::VeryHigh);
        let lat = Output::new(pa3, Level::High, Speed::VeryHigh);
        let rst = Output::new(pa4, Level::High, Speed::VeryHigh);
        let sck = Output::new(pa5, Level::High, Speed::VeryHigh);
        let sda = Output::new(pa6, Level::High, Speed::VeryHigh);
        let rows = [
            Output::new(pa7, Level::High, Speed::VeryHigh),
            Output::new(pa15, Level::High, Speed::VeryHigh),
            Output::new(pb0, Level::High, Speed::VeryHigh),
            Output::new(pb1, Level::High, Speed::VeryHigh),
            Output::new(pb2, Level::High, Speed::VeryHigh),
            Output::new(pc3, Level::High, Speed::VeryHigh),
            Output::new(pc4, Level::High, Speed::VeryHigh),
            Output::new(pc5, Level::High, Speed::VeryHigh)];
        return Self {
            sb,
            lat,
            rst,
            sck,
            sda,
            rows,
        };
    }

    /// Make a brief high pulse of the SCK pin
    fn pulse_sck(&mut self) {
        self.sck.set_high();
        // for _ in 0..2 {
        //     cortex_m::asm::nop();
        // }
        Delay.delay_us(10);
        self.sck.set_low();
    }
    /// Make a brief low pulse of the LAT pin
    fn pulse_lat(&mut self) {
        self.lat.set_high();
        // for _ in 0..2 {
        //     cortex_m::asm::nop();
        // }
        Delay.delay_us(10);
        self.lat.set_low();
    }

    /// Send a byte on SDA starting with the MSB and pulse SCK high after each bit
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

    /// Send a full row of bytes in BGR order and pulse LAT low. Gamma correction
    /// must be applied to every pixel before sending them. The previous row must
    /// be deactivated and the new one activated.
    pub fn send_row(&mut self, row: usize, pixels: &[Color]) {
        // Deactivate the previous row
        for r in self.rows.iter_mut() {
            r.set_low();
        }
        // Send the new row
        for pixel in pixels {
            let pixel = pixel.gamma_correct();
            self.send_byte(pixel.b);
            self.send_byte(pixel.g);
            self.send_byte(pixel.r);
        }
        // Pulse LAT
            self.pulse_lat();
        // Activate the new row
        if let Some(row_pin) = self.rows.get_mut(row) {
            row_pin.set_high();
        }
    }

    /// Initialize bank0 by temporarily setting SB to low and sending 144 one bits,
    /// pulsing SCK high after each bit and pulsing LAT low at the end. SB is then
    /// restored to high.
    fn init_bank0(&mut self) {
        self.sb.set_low();
        /// TODO: Send 144 one bits to be checked...
        for _ in 0..18 {
            self.send_byte(0xFF);
        }
        self.pulse_lat();
        self.sb.set_high();
    }

    /// Display a full image, row by row, as fast as possible.
    pub fn display_image(&mut self, image: &Image) {
        // Do not forget that image.row(n) gives access to the content of row n,
        // and that self.send_row() uses the same format.
        self.init_bank0();
        for row in 0..8 {
            self.send_row(row, &image.row(row));
        }
    }
}






