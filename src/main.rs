#![no_main]
#![no_std]

use core::fmt::Write;
use panic_halt as _;

use hal::spi::*;

use stm32f0xx_hal as hal;
use stm32f0xx_hal::delay::Delay;

use crate::hal::{pac, prelude::*};

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init_print};

use embedded_graphics::{
	mono_font::{ascii::FONT_6X9, MonoTextStyle},
	pixelcolor::BinaryColor,
	prelude::*,
	primitives::{Circle, PrimitiveStyle},
	text::Text,
};
use st7920::ST7920;

use heapless::String;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	let mut x = 0;
	if let Some(mut p) = pac::Peripherals::take() {
		let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

		let core = cortex_m::Peripherals::take().unwrap();

		let gpioc = p.GPIOC.split(&mut rcc);

		// (Re-)configure PC8 as output
		let mut blue_led = cortex_m::interrupt::free(|cs| gpioc.pc8.into_push_pull_output(cs));
		let mut green_led = cortex_m::interrupt::free(|cs| gpioc.pc9.into_push_pull_output(cs));

		let mut delay = Delay::new(core.SYST, &rcc);

		rprintln!("Hello, world!");

		blue_led.toggle().ok();

		// Display init
		let gpioa = p.GPIOA.split(&mut rcc);
		let sck = cortex_m::interrupt::free(|cs| gpioa.pa5.into_alternate_af0(cs));
		let mosi = cortex_m::interrupt::free(|cs| gpioa.pa7.into_alternate_af0(cs));
		let _miso = cortex_m::interrupt::free(|cs| gpioa.pa6.into_alternate_af0(cs));
		let reset = cortex_m::interrupt::free(|cs| gpioa.pa1.into_push_pull_output(cs));
		let cs = cortex_m::interrupt::free(|cs| gpioa.pa2.into_push_pull_output(cs));
		let spi = Spi::spi1(
			p.SPI1,
			(sck, _miso, mosi),
			Mode {
				polarity: Polarity::IdleLow,
				phase: Phase::CaptureOnFirstTransition,
			},
			1_000_000.hz(),
			&mut rcc,
		);

		let mut disp = ST7920::new(spi, reset, Some(cs), false);
		disp.init(&mut delay).expect("could not init display");
		disp.clear(&mut delay).expect("could not clear display");

		let c = Circle::new(Point::new(20, 20), 20)
			.into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
		let hello = Text::new(
			"Hello Rust!",
			Point::new(40, 16),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);

		c.draw(&mut disp).unwrap();
		hello.draw(&mut disp).unwrap();
		disp.flush(&mut delay).expect("could not flush display");

		loop {
			blue_led.toggle().ok();
			green_led.toggle().ok();
			delay.delay_ms(200u16);
			x += 1;

			let mut s: String<150> = String::from("");
			write!(&mut s, "X = {:?}", x);

			let t = Text::new(
				s.as_str(),
				Point::new(40, 26),
				MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
			);

			disp.clear_buffer();
			c.draw(&mut disp).unwrap();
			hello.draw(&mut disp).unwrap();
			t.draw(&mut disp).unwrap();
			disp.flush(&mut delay).expect("could not flush display");

			rprintln!("X = {:?}", x);
		}
	}

	loop {
		continue;
	}
}
