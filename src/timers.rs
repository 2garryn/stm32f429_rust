use stm32f4::stm32f429;

use stm32f429::interrupt;
use core::ops::Fn;
use core::sync::atomic::{AtomicI32, Ordering};


pub static T1: AtomicI32 = AtomicI32::new(-1);
pub static T2: AtomicI32 = AtomicI32::new(-1);
pub static T3: AtomicI32 = AtomicI32::new(-1);
pub static T4: AtomicI32 = AtomicI32::new(-1);


pub fn init(tim: stm32f429::TIM6) {
    tim.psc.write(|w| w.psc().bits(90-1));
    tim.arr.write(|w| w.arr().bits(1000-1));
    tim.dier.modify(|_, w| w.uie().enabled());
    tim.cr1.modify(|_, w| w.cen().enabled());
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
        if let 0 = self.at.load(Ordering::Relaxed) {
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


#[interrupt]
fn TIM6_DAC() {
    let f = |t| {
        if t > 0 { Some(t - 1) }
        else { None }
    };
    T1.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    T2.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    T3.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    T4.fetch_update(Ordering::Relaxed, Ordering::Relaxed, f);
    ()
}
