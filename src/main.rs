#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;
use stm32f0xx_hal::delay::Delay;

use crate::hal::{pac, prelude::*};

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init_print};

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
		loop {
			// Turn PC81 on a million times in a row
			// for _ in 0..1_000_000 {
			// 	led.set_high().ok();
			// }
			// // Then turn PA1 off a million times in a row
			// for _ in 0..1_000_000 {
			// 	led.set_low().ok();
			// }

			blue_led.toggle().ok();
			green_led.toggle().ok();
			delay.delay_ms(200u16);
			x += 1;
			rprintln!("X = {:?}", x);
		}
	}

	loop {
		continue;
	}
}
