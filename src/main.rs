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

use smart_leds::{brightness, hsv::RGB8, SmartLedsWrite, RGB};
use smart_leds::{gamma, hsv::hsv2rgb, hsv::Hsv};
use ws2812_timer_delay as ws2812;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	let mut x = 0;
	if let Some(p) = pac::Peripherals::take() {
		let rcc = p.RCC.constrain();
		let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(50.MHz()).freeze();
		//let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

		let mut delay = p.TIM1.delay_us(&clocks);

		let gpioc = p.GPIOC.split();

		// (Re-)configure PC13 as output
		let mut blue_led = gpioc.pc13.into_push_pull_output();

		rprintln!("Hello, world!");

		blue_led.toggle();

		//RGB strip init
		let gpiob = p.GPIOB.split();

		let mut strip_pin = gpiob.pb15.into_push_pull_output();
		let mut timer = p.TIM2.counter_hz(&clocks);
		timer.start(3200.kHz()).unwrap();
		let mut ws = ws2812::Ws2812::new(timer, strip_pin);
		const NUM_LEDS: usize = 84;
		let mut data = [RGB8 { r: 0, g: 0, b: 0 }; NUM_LEDS];
		// Wait before start write for syncronization
		delay.delay(200.micros());
		//delay.delay_ms(400u16);

		// Display init
		let gpioa = p.GPIOA.split();
		let reset = gpioa.pa4.into_push_pull_output();
		let cs = gpioa.pa6.into_push_pull_output();

		let spi = p.SPI1.spi(
			(gpioa.pa5, NoPin, gpioa.pa7),
			Mode {
				polarity: Polarity::IdleLow,
				phase: Phase::CaptureOnFirstTransition,
			},
			600_000.Hz(),
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
			// rgb

			// for j in 0..NUM_LEDS {
			// 	for i in 0..NUM_LEDS {
			// 		let mut color_on = RGB8 { r: 64, g: 0, b: 0 };
			// 		let mut color_off = RGB8 { r: 0, g: 0, b: 0 };
			// 		let mut color_target = RGB8 { r: 0, g: 32, b: 0 };
			// 		let mut color_target_on = RGB8 {
			// 			r: 127,
			// 			g: 32,
			// 			b: 0,
			// 		};
			//
			// 		if i == j {
			// 			data[NUM_LEDS - i - 1] = color_on;
			// 		} else {
			// 			data[NUM_LEDS - i - 1] = color_off;
			// 		}
			//
			// 		if (i >= 26 && i <= 36) {
			// 			data[NUM_LEDS - i - 1] = color_target;
			// 			if i == j {
			// 				data[NUM_LEDS - i - 1] = color_target_on;
			// 			}
			// 		}
			// 	}
			// 	// before writing, apply gamma correction for nicer rainbow
			// 	ws.write(gamma(data.iter().cloned())).unwrap();
			// 	delay.delay_ms(5u16);
			// }

			blue_led.toggle();
			delay.delay_ms(500u16);
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
