use stm32f4::stm32f429;

use stm32f429::interrupt;
use core::ops::Fn;
use core::sync::atomic::{AtomicI32, Ordering};
use core::cell::RefCell;


pub static T1: AtomicI32 = AtomicI32::new(-1);
pub static T2: AtomicI32 = AtomicI32::new(-1);
pub static T3: AtomicI32 = AtomicI32::new(-1);
pub static T4: AtomicI32 = AtomicI32::new(-1);


static mut TIM6: RefCell<Option<stm32f429::TIM6>> = RefCell::new(None);

pub fn init(tim: stm32f429::TIM6) {
    unsafe { 
        tim.psc.write(|w| w.psc().bits(90-1));
        tim.arr.write(|w| w.arr().bits(1000-1));
        tim.dier.modify(|_, w| w.uie().enabled());
        tim.cr1.modify(|_, w| w.cen().enabled());
        stm32f429::NVIC::unmask(stm32f429::interrupt::TIM6_DAC);
        TIM6.replace(Some(tim));
    };

}


fn acquire() -> Option<&'static AtomicI32> {
    if let -1 = T1.load(Ordering::Relaxed) {
        return Some(&T1);
    };
    if let -1 = T2.load(Ordering::Relaxed) {
        return Some(&T2);
    };    
    if let -1 = T3.load(Ordering::Relaxed) {
        return Some(&T3);
    };    
    if let -1 = T4.load(Ordering::Relaxed) {
        return Some(&T4);
    };
    None
}


pub struct Once<T: Fn() -> ()> {
    clb: T,
    at: &'static AtomicI32
}

impl <T: Fn() -> ()>Once<T> {
    pub fn main_loop(&self) {
        if self.at.load(Ordering::Relaxed) == 0 {
            self.at.store(-1, Ordering::Relaxed);
            (self.clb)();
        }
    }
}

pub fn new_once<T>(ms: u16, clb: T) -> Option<Once<T>> 
    where T: Fn() -> () {
    if let Some(at) = acquire() {
        at.store(i32::from(ms), Ordering::Relaxed);
        return Some(Once{
            clb: clb,
            at: at
        }) 
    }
    None
}


pub struct Repeat<T: Fn(&mut S, &mut RepeatHandler) -> (), S> {
    clb: T,
    at: Option<&'static AtomicI32>,
    ms: u16,
    state: S,
    stopped: bool
}


impl <T: Fn(&mut S, &mut RepeatHandler) -> (), S> Repeat<T, S> {
    pub fn main_loop(&mut self) {
        if let Some(at) = self.at {
            if at.load(Ordering::Relaxed) == 0 {
                let mut handler = RepeatHandler{
                    ms: 0, 
                    ms_updated: false, 
                    stopped: false
                };
                (self.clb)(&mut self.state, &mut handler);
                if handler.stopped {
                    self.stop();
                    return
                }
                if handler.ms_updated {
                    self.ms = handler.ms;
                }
                at.store(i32::from(self.ms), Ordering::Relaxed);
            }
        }
    }
    pub fn stop(&mut self) {
        if let Some(at) = self.at {
            at.store(-1, Ordering::Relaxed);
            self.at = None
        }
    }
}





pub fn new_repeat<T, S>(ms: u16, clb: T, state: S) -> Option<Repeat<T, S>> 
    where T: Fn(&mut S, &mut RepeatHandler) -> () {
    if let Some(at) = acquire() {
        at.store(i32::from(ms), Ordering::Relaxed);
        return Some(Repeat{
            clb: clb,
            at: Some(at),
            ms: ms,
            state: state,
            stopped: false
        }) 
    }
    None
}


pub struct RepeatHandler {
    ms: u16,
    stopped: bool,
    ms_updated: bool
}

impl RepeatHandler {
    pub fn update_ms(&mut self, ms: u16) {
        self.ms = ms;
        self.ms_updated = true
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
}

#[interrupt]
fn TIM6_DAC() {
    unsafe {
        let tim6 = TIM6.borrow();
        tim6.as_ref().unwrap().sr.modify(|_, w| w.uif().clear_bit());
    }
    
    let f = |t| {
        if t > 0 { Some(t - 1) }
        else { None }
    };
    let _ = T1.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    let _ = T2.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    let _ = T3.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    let _ = T4.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    
    ()
}
