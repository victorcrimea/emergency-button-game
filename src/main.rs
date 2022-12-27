#![no_main]
#![no_std]

use core::fmt::Write;
use panic_halt as _;

use stm32f4xx_hal as hal;

use crate::hal::{gpio::NoPin, pac, prelude::*};

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
use stm32f4xx_hal::spi::{Mode, Phase, Polarity};

#[entry]
fn main() -> ! {
	rtt_init_print!();
	let mut x = 0;
	if let Some(p) = pac::Peripherals::take() {
		let rcc = p.RCC.constrain();
		//let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
		let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(50.MHz()).freeze();

		let mut delay = p.TIM1.delay_us(&clocks);

		let gpioc = p.GPIOC.split();

		// (Re-)configure PC13 as output
		let mut blue_led = gpioc.pc13.into_push_pull_output();

		rprintln!("Hello, world!");

		blue_led.toggle();

		// Display init
		let gpioa = p.GPIOA.split();
		// let sck = cortex_m::interrupt::free(|cs| gpioa.pa5.into_alternate_af0(cs));
		// let mosi = cortex_m::interrupt::free(|cs| gpioa.pa7.into_alternate_af0(cs));
		// let _miso = cortex_m::interrupt::free(|cs| gpioa.pa6.into_alternate_af0(cs));
		let reset = gpioa.pa4.into_push_pull_output();
		let cs = gpioa.pa2.into_push_pull_output();
		// let spi = Spi::spi1(
		// 	p.SPI1,
		// 	(sck, _miso, mosi),
		// 	Mode {
		// 		polarity: Polarity::IdleLow,
		// 		phase: Phase::CaptureOnFirstTransition,
		// 	},
		// 	1_000_000.hz(),
		// 	&mut rcc,
		// );

		let spi = p.SPI1.spi(
			(gpioa.pa5, NoPin, gpioa.pa7),
			Mode {
				polarity: Polarity::IdleLow,
				phase: Phase::CaptureOnFirstTransition,
			},
			300_000.Hz(),
			&clocks,
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
			blue_led.toggle();
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
