#![no_std]
#![no_main]

// Panic provider crate
use panic_persist as _;

// Used to set the program entry point
use cortex_m_rt::entry;

extern crate feather_m4 as hal;

use hal::prelude::*;
use hal::clock::GenericClockController;
use hal::pac::{CorePeripherals,Peripherals};
use hal::trng::Trng;
use hal::delay::Delay;
use hal::Pins;
use hal::{i2c_master,spi_master};
use hal::time::U32Ext;

use rtt_target::{rprintln, rtt_init_print};

use embedded_graphics::{
    fonts::{Font8x16, Text},
    pixelcolor::Rgb565,
    prelude::*,
    style::TextStyleBuilder,
};

use bbq10kbd::{Bbq10Kbd, KeyRaw};

mod buffer;

use ili9341::{Ili9341, Orientation};

#[entry]
fn main() -> ! {
    match inner_main() {
        Ok(()) => cortex_m::peripheral::SCB::sys_reset(),
        Err(e) => panic!(e),
    }
}

fn inner_main() -> Result<(), &'static str> {
    let mut peripherals = Peripherals::take().ok_or("Error getting board!")?;
    let mut _pins = Pins::new(peripherals.PORT);
    let _core = CorePeripherals::take().unwrap();

    let mut _rng: Trng = Trng::new(&mut peripherals.MCLK, peripherals.TRNG);
    let mut _clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(_core.SYST, &mut _clocks);

    // use ChannelMode::NoBlockS
    rtt_init_print!(NoBlockSkip, 4096);

    if let Some(msg) = panic_persist::get_panic_message_utf8() {
        rprintln!("{}", msg);
    } else {
        rprintln!("Clean boot!");
    }

    let kbd_lcd_reset = _pins.d5;
    let _stm_cs = _pins.d6;
    let lcd_cs = _pins.d9;
    let lcd_dc = _pins.d10;

    // i2c keyboard interface
    // kbd SDA = D12
    // kbd SCL = D11
    // FREQ 100KHZ
    let kbd_i2c = i2c_master(
        &mut _clocks,
        100u32.khz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        _pins.sda,
        _pins.scl,
        &mut _pins.port
    );

    let mut kbd = Bbq10Kbd::new(kbd_i2c);

    let lcd_spi = spi_master(
        &mut _clocks,
        32u32.mhz(),
        peripherals.SERCOM1,
        &mut peripherals.MCLK,
        _pins.sck,
        _pins.mosi,
        _pins.miso,
        &mut _pins.port
    );

    let mut lcd = Ili9341::new_spi(
        lcd_spi,
        lcd_cs.into_push_pull_output(&mut _pins.port),
        lcd_dc.into_push_pull_output(&mut _pins.port),
        kbd_lcd_reset.into_push_pull_output(&mut _pins.port),
        &mut delay,
    ).unwrap();

    lcd.set_orientation(Orientation::Landscape).unwrap();

    let mut _buffy = [0u16; 24 * 32];
    let mut buffy2 = [[0u16; 320]; 240];

    let mut fbuffy = buffer::FrameBuffer::new(&mut buffy2);

    // //                                     rrrrr gggggg bbbbb
    // buffy.iter_mut().for_each(|px| *px = 0b11111_000000_00000);

    let mut style = TextStyleBuilder::new(Font8x16)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    kbd.set_backlight(255).unwrap();

    let vers = kbd.get_version().unwrap();

    rprintln!("Vers: {:?}", vers);

    kbd.sw_reset().unwrap();
    delay.delay_ms(10u8);

    let vers = kbd.get_version().unwrap();

    rprintln!("Vers: {:?}", vers);

    let mut cursor = Cursor { x: 0, y: 0 };

    lcd.clear(Rgb565::BLACK).map_err(|_| "Fade to error")?;
    fbuffy.clear(Rgb565::BLACK).map_err(|_| "Fade to error")?;

    loop {
        let key = kbd.get_fifo_key_raw().map_err(|_| "bad fifo")?;

        match key {
            // LL
            KeyRaw::Pressed(6) => {
                style = TextStyleBuilder::new(Font8x16)
                    .text_color(Rgb565::WHITE)
                    .background_color(Rgb565::BLACK)
                    .build();
            }
            // LR
            KeyRaw::Pressed(17) => {
                style = TextStyleBuilder::new(Font8x16)
                    .text_color(Rgb565::RED)
                    .background_color(Rgb565::BLACK)
                    .build();
            }
            // RL
            KeyRaw::Pressed(7) => {
                style = TextStyleBuilder::new(Font8x16)
                    .text_color(Rgb565::GREEN)
                    .background_color(Rgb565::BLACK)
                    .build();
            }
            // RR
            KeyRaw::Pressed(18) => {
                style = TextStyleBuilder::new(Font8x16)
                    .text_color(Rgb565::BLUE)
                    .background_color(Rgb565::BLACK)
                    .build();
            }
            // Up
            KeyRaw::Pressed(1) => {
                cursor.up();
            }
            // Down
            KeyRaw::Pressed(2) => {
                cursor.down();
            }
            // Left
            KeyRaw::Pressed(3) => {
                cursor.left();
            }
            // Right
            KeyRaw::Pressed(4) => {
                cursor.right();
            }
            // Center
            KeyRaw::Pressed(5) => {
                kbd.sw_reset().unwrap();
                cursor = Cursor { x: 0, y: 0 };
                fbuffy.clear(Rgb565::BLACK).map_err(|_| "Fade to error")?;
            }
            // Backspace
            KeyRaw::Pressed(8) => {
                cursor.left();
                Text::new(" ", cursor.pos())
                    .into_styled(style)
                    .draw(&mut fbuffy)
                    .map_err(|_| "bad lcd")?;
            }
            // Enter
            KeyRaw::Pressed(10) => {
                cursor.enter();
            }
            KeyRaw::Pressed(k) => {
                rprintln!("Got key {}", k);
                if let Ok(s) = core::str::from_utf8(&[k]) {
                    Text::new(s, cursor.pos())
                        .into_styled(style)
                        .draw(&mut fbuffy)
                        .map_err(|_| "bad lcd")?;

                    cursor.right();
                }
            }
            KeyRaw::Invalid => {
                if let Some(buf) = fbuffy.inner() {
                    // timer.start(1_000_000u32);
                    lcd.draw_raw(0, 0, 319, 239, buf).map_err(|_| "bad buffy")?;
                    // let done = timer.read();
                    // rprintln!("Drew in {}ms.", done / 1000);
                } else {
                    delay.delay_ms(38u8);
                }
            }
            _ => {}
        }
    }
}

struct Cursor {
    x: i32,
    y: i32,
}

impl Cursor {
    fn up(&mut self) {
        self.y -= 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn down(&mut self) {
        self.y += 1;
        if self.y >= 15 {
            self.y = 14;
        }
    }

    fn left(&mut self) {
        self.x -= 1;
        if self.x < 0 {
            if self.y != 0 {
                self.x = 39;
                self.up();
            } else {
                self.x = 0;
            }
        }
    }

    fn right(&mut self) {
        self.x += 1;
        if self.x >= 40 {
            self.x = 0;
            self.down();
        }
    }

    fn enter(&mut self) {
        if self.y != 14 {
            self.x = 0;
            self.down();
        }
    }

    fn pos(&self) -> Point {
        Point::new(self.x * 8, self.y * 16)
    }
}
