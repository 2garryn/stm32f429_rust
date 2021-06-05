#![no_main]
#![no_std]

#[cfg(debug_assertions)]
use panic_halt as _;


use stm32f4::stm32f429;
use cortex_m_rt::entry;

use clocking::*;
//use timers;

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
    timers::init(periph.TIM6);

    
    let gpiog = &periph.GPIOG;
    gpiog.moder.modify(|_, w| w.moder13().output());
    gpiog.otyper.modify(|_, w| w.ot13().push_pull());
    gpiog.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
    gpiog.odr.modify(|_, w| w.odr13().high());


    let mut t1 = timers::new_once(2000, || {
        gpiog.odr.modify(|_, w| w.odr13().low());
    });

    let mut t2 = timers::new_once(2000, || {
        gpiog.odr.modify(|_, w| w.odr13().high());
    });

    loop {
        t1.take().unwrap().main_loop();
        t2.take().unwrap().main_loop();

        continue;
    }
}

