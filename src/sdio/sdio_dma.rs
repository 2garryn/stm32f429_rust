
use stm32f4::stm32f429;
use crate::sdio::CardError;
/*
SDIO
DMA2
a) Stream3 Channel4
b) Stream6 Channel4
*/
const STREAM3: usize = 3;



pub struct SdioDma<'a> {
    dma: &'a stm32f429::DMA2
}


impl<'a> SdioDma<'a>  {
    pub fn new(dma: &'a stm32f429::DMA2) -> Self {
        SdioDma {
            dma: dma
        }
    }

    pub fn init(&self) {
        let fifo_ptr = stm32f429::SDIO::ptr() as u32 + 0x80;
        self.dma.st[STREAM3].par.write(|w| unsafe { w.pa().bits(fifo_ptr) });
            // use channel 4
        self.dma.st[STREAM3].cr.write(|w| w.chsel().bits(4)
            .mburst().incr4()
            .pburst().incr4()
            .pl().very_high()
            .msize().bits32()
            .psize().bits32()
            .minc().set_bit()
            .pinc().clear_bit()
            .pfctrl().set_bit());
        self.dma.st[STREAM3].fcr.write(|w| w.dmdis().set_bit().fth().full());
        self.dma.st[STREAM3].cr.modify(|_r, w| w.dir().peripheral_to_memory());
    }

    pub fn p2m(&self, buf: &mut [u8]) {
        self.dma.st[STREAM3].cr.modify(|_r, w| w.en().disabled());
            // clean stream 3 interruption flag
        let clean_flag = 0b111101 << 22;
        self.dma.lifcr.write(|w| unsafe { w.bits(clean_flag) });

        // set dir from memory to peripheral, set buffer ptr, set buffer length

        self.dma.st[STREAM3].m0ar.write(|w| unsafe { w.m0a().bits(buf.as_ptr() as u32) });
        //let ndtr = buf.len() / 4; 
        //self.dma.st[STREAM3].ndtr.modify(|_, w| w.ndt().bits(ndtr as u16));
        self.dma.st[STREAM3].cr.modify(|_r, w| w.en().enabled());

    }

    pub fn p2m_completed(&self) -> bool {
        self.dma.lisr.read().tcif3().bits()
    }
}