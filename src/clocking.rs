use stm32f4::stm32f429;
pub struct Clocking<'a> {
    rcc: &'a mut  stm32f429::RCC
}

impl<'a> Clocking<'a> {
    pub fn new(rcc: &mut stm32f429::RCC) -> Clocking {
        Clocking {
            rcc: rcc
        }
    }

    pub fn init(&self, flash: &mut stm32f429::FLASH) {
        self.rcc.cr.modify(|_, w| w.hseon().set_bit());
        while self.rcc.cr.read().hserdy().bit_is_clear() {}
    
        self.rcc.pllcfgr.modify(|_, w| w.pllsrc().hse());
    
        self.rcc.cr.modify(|_, w| w.pllon().clear_bit());
    
        self.rcc.pllcfgr.modify(|_, w| unsafe { 
            w.pllp().div2()
            .plln().bits(360)
            .pllm().bits(8)
            .pllq().bits(8)
        });
    
        self.rcc.cfgr.modify(|_, w| w
            .hpre().div1()
            .ppre1().div4()
            .ppre2().div2());
            
        //PLL enable
        self.rcc.cr.modify(|_, w| w.pllon().set_bit());
    
        // Wait PLL is ready	
        while self.rcc.cr.read().pllrdy().bit_is_clear() {}
    
        flash.acr.modify(|_, w|  unsafe {
            w.prften().set_bit()
            .icen().set_bit()
            .dcen().set_bit()
            .latency().bits(5)
        });
    
        self.rcc.cfgr.modify(|_, w| w.sw().pll());
        while !self.rcc.cfgr.read().sws().is_pll() {}
    }

    pub fn gpiod_enable(&self)  {
        self.rcc.ahb1enr.modify(|_, w| w.gpioden().enabled());
    }
    pub fn gpiog_enable(&self)  {
        self.rcc.ahb1enr.modify(|_, w| w.gpiogen().enabled());
    }
}
