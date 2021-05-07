#![no_main]
#![no_std]

use panic_halt as _;
use stm32f4::stm32f429;
use cortex_m_rt::entry;

use clocking::*;
use delay::*;

mod clocking;
mod timers;
mod logger;

fn do_something1() {

}

fn do_something2() {

}

#[entry]
fn main() -> ! {
    let mut periph = stm32f429::Peripherals::take().unwrap();
    let cl = Clocking::new(&mut periph.RCC);
    cl.init(&mut periph.FLASH);

    //let mut dl = Timers1::new(&mut periph.TIM6);
    //dl.add(0, do_something1);
    //dl.add(1, do_something2);
    //dl.add(2, cl.gpiog_enable );

    //cl.gpiog_enable();
    
    let gpiog = &periph.GPIOG;
    gpiog.moder.modify(|_, w| w.moder13().output());
    gpiog.otyper.modify(|_, w| w.ot13().push_pull());
    gpiog.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
    gpiog.odr.modify(|_, w| w.odr13().high());

    loop {
     //   dl.do_loop();
        continue;
    }
}

