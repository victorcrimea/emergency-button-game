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

	if let Some(p) = pac::Peripherals::take() {
		let rcc = p.RCC.constrain();
		let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(84.MHz()).freeze();
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
		let cs = gpioa.pa2.into_push_pull_output();
		//let mut buzzer = gpiob.pb12.into_push_pull_output();

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

		let mut button_right = gpioa.pa11.into_pull_up_input();
		let mut button_left = gpioa.pa8.into_pull_up_input();
		let mut button_restart = gpioa.pa1.into_pull_up_input();

		let player1 = Text::new(
			"Left",
			Point::new(1, 5),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);

		let player2 = Text::new(
			"Right",
			Point::new(99, 5),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);

		player1.draw(&mut disp).unwrap();
		player2.draw(&mut disp).unwrap();

		let mut score_left = 0;
		let mut score_right = 0;
		let mut rounds = 0;
		let mut dead = 0;
		let mut level = 0;
		let mut player1_early = false;
		let mut player2_early = false;
		let target_start = 31;
		let target_end = 41;

		let mut rounds_string: String<80> = String::from("Rounds: 0");
		let mut score_left_string: String<20> = String::from("0000");
		let mut score_right_string: String<20> = String::from("0000");
		let game_over_string: String<40> = String::from("Game Over!");
		let mut level_string: String<40> = String::from("Level: 0");
		let player_1_early_string: String<40> = String::from("Early!");
		let player_2_early_string: String<40> = String::from("Early!");

		let rounds_text = Text::new(
			rounds_string.as_str(),
			Point::new(55, 60),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);
		rounds_text.draw(&mut disp).unwrap();

		let level_text = Text::new(
			level_string.as_str(),
			Point::new(1, 60),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);
		level_text.draw(&mut disp).unwrap();

		disp.flush(&mut delay).expect("could not flush display");

		let player_1_early_text = Text::new(
			player_1_early_string.as_str(),
			Point::new(1, 36),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);

		let player_2_early_text = Text::new(
			player_2_early_string.as_str(),
			Point::new(90, 36),
			MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
		);

		let speeds: [(u8); 9] = [(40), (30), (20), (10), (10), (10), (10), (10), (10)];
		let rounds_per_level = 10;
		let mut rounds_in_level = rounds_per_level;

		'main_loop: loop {
			// rgb
			rounds_in_level -= 1;
			if rounds_in_level == 0 && level < 8 {
				level += 1;
				rounds_in_level = rounds_per_level;
			}

			'led_outer: for j in 0..NUM_LEDS {
				if (j == NUM_LEDS - 1 - dead) {
					dead += 1;
					//buzzer.set_high();
					//delay.delay_ms(200u16);
					//buzzer.set_low();
				}

				if (j >= target_start && j <= target_end) {
					if !player2_early {
						if button_right.is_low() {
							score_right += 1;
							loop {
								if button_right.is_high() {
									break;
								}
								continue;
							}
							break 'led_outer;
						}
					}
					if !player1_early {
						if button_left.is_low() {
							score_left += 1;
							loop {
								if button_left.is_high() {
									break;
								}
								continue;
							}
							break 'led_outer;
						}
					}
				}

				if j < target_start {
					if !player2_early {
						if button_right.is_low() {
							player2_early = true;
							score_right -= 1;
							player_2_early_text.draw(&mut disp).unwrap();
							disp.flush(&mut delay).expect("could not flush display");
						}
					}

					if !player1_early {
						if button_left.is_low() {
							player1_early = true;
							score_left -= 1;
							player_1_early_text.draw(&mut disp).unwrap();
							disp.flush(&mut delay).expect("could not flush display");
						}
					}
				}

				for i in 0..NUM_LEDS {
					let mut color_on = RGB8 { r: 64, g: 0, b: 0 };
					let mut color_off = RGB8 { r: 0, g: 0, b: 0 };
					let mut color_target = RGB8 { r: 0, g: 32, b: 0 };
					let mut color_dead = RGB8 { r: 64, g: 0, b: 0 };
					let mut color_target_on = RGB8 {
						r: 127,
						g: 32,
						b: 0,
					};

					if i == j {
						data[NUM_LEDS - i - 1] = color_on;
					} else {
						data[NUM_LEDS - i - 1] = color_off;
					}

					if (i >= target_start && i <= target_end) {
						data[NUM_LEDS - i - 1] = color_target;
						if i == j {
							data[NUM_LEDS - i - 1] = color_target_on;
						}
					}

					if i > NUM_LEDS - 1 - dead {
						data[NUM_LEDS - i - 1] = color_dead;
					}
				}

				// before writing, apply gamma correction for nicer rainbow
				ws.write(gamma(data.iter().cloned())).unwrap();
				delay.delay_ms(speeds[level]);
			}

			blue_led.toggle();
			//delay.delay_ms(500u16);
			rounds += 1;

			rounds_string.clear();
			level_string.clear();
			score_left_string.clear();
			score_right_string.clear();

			write!(&mut rounds_string, "Rounds: {:?}", rounds);
			write!(&mut level_string, "Level: {:?}", level);
			write!(&mut score_left_string, "{:?}", score_left);
			write!(&mut score_right_string, "{:?}", score_right);

			let rounds_text = Text::new(
				rounds_string.as_str(),
				Point::new(55, 60),
				MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
			);

			let level_text = Text::new(
				level_string.as_str(),
				Point::new(1, 60),
				MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
			);

			let score_left_text = Text::new(
				score_left_string.as_str(),
				Point::new(1, 14),
				MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
			);

			let score_right_text = Text::new(
				score_right_string.as_str(),
				Point::new(99, 14),
				MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
			);

			disp.clear_buffer();
			player1.draw(&mut disp).unwrap();
			player2.draw(&mut disp).unwrap();

			rounds_text.draw(&mut disp).unwrap();
			level_text.draw(&mut disp).unwrap();

			score_left_text.draw(&mut disp).unwrap();
			score_right_text.draw(&mut disp).unwrap();

			if player1_early {
				player1_early = false;
			}

			if player2_early {
				player2_early = false;
			}

			if (dead == 42) {
				let game_over_text = Text::new(
					game_over_string.as_str(),
					Point::new(35, 46),
					MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
				);
				game_over_text.draw(&mut disp).unwrap();
				disp.flush(&mut delay).expect("could not flush display");
				loop {
					if button_restart.is_low() {
						rounds = 0;
						score_left = 0;
						score_right = 0;
						dead = 0;
						level = 0;
						rounds_in_level = rounds_per_level;
						player1_early = false;
						player2_early = false;
						break;
					}
					continue;
				}
			}

			disp.flush(&mut delay).expect("could not flush display");

			rprintln!("X = {:?}", rounds);
		}
	}

	loop {
		continue;
	}
}
