#![no_main]
#![no_std]

#[cfg(debug_assertions)]
use panic_halt as _;


use stm32f4::stm32f429;
use cortex_m_rt::entry;
use cortex_m;

use clocking::*;
//use timers;

mod clocking;
mod timers;
mod logger;
mod sdio;


fn do_something1() {

}

fn do_something2() {

}

#[entry]
fn main() -> ! {
    let mut periph = stm32f429::Peripherals::take().unwrap();
    let cl = Clocking::new(&mut periph.RCC);
    cl.init(&mut periph.FLASH);

    cl.gpiod_enable();
    cl.gpioc_enable();
    cl.sdio_enable();
    let res = sdio::new(periph.SDIO, &mut periph.GPIOD, &mut periph.GPIOC);
   // let res = sd.init(&mut periph.GPIOD, &mut periph.GPIOC);
    match res {
        Ok(c) => {
            cl.gpiog_enable();
            let gpiog = &periph.GPIOG;
            gpiog.moder.modify(|_, w| w.moder13().output());
            gpiog.otyper.modify(|_, w| w.ot13().push_pull());
            gpiog.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
            gpiog.odr.modify(|r, w| {
                    w.odr13().high()
            });
        },
        _ => {
            /*
            cl.gpiog_enable();
            let gpiog = &periph.GPIOG;
            gpiog.moder.modify(|_, w| w.moder13().output());
            gpiog.otyper.modify(|_, w| w.ot13().push_pull());
            gpiog.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
            gpiog.odr.modify(|r, w| {
                    w.odr13().high()
            });
            */
        }
    }


    


/*

    timers::init(periph.TIM6);
    let mut repeat = timers::new_repeat(1000, |state, handler| {
        if *state == 6 {
            handler.update_ms(250);
        }
        if *state == 12 {
            handler.stop();
        }
        *state+=1;
        gpiog.odr.modify(|r, w| {
            if r.odr13().is_low() {
                w.odr13().high()
            } else {
                w.odr13().low()
            }
        });
    }, 1).unwrap();
    */
    loop {
      //  repeat.main_loop();
        continue;
    }
    
}

