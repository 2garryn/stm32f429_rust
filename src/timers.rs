use stm32f4::stm32f429;

use stm32f429::interrupt;



static mut TIMERS: Timers = Timers {
    tim6: Some(stm32f429::Peripherals::take().unwrap().TIM6),
};

pub struct Timers {
    tim6:  Option<stm32f429::TIM6>
}

impl Timers {
    pub fn do_loop(&self) {}


}

#[interrupt]
fn TIM6_DAC() {

}

