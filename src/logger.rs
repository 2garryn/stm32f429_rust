#![allow(dead_code)]

pub trait HWLogger {
    fn write(b: u8);
    fn bytes_one_time() -> u8;
}


pub struct Log<THWLogger: HWLogger> {
    hwl: THWLogger,
    buf: [u8; 4096],
    wi: usize,
    ri: usize
}

impl<THWLogger: HWLogger> Log<THWLogger> {
    pub fn new(hwl: THWLogger) -> Log<THWLogger> {
        Log {
            buf: [0; 4096],
            wi: 0,
            ri: 0,
            hwl: hwl
        }
    }
    pub fn info(&mut self, s: &[u8]) {
        self.str_to_buff(b"I:");
        self.str_to_buff(s);
    }
    pub fn error(&mut self, s: &[u8]) {
        self.str_to_buff(b"E:");
        self.str_to_buff(s);
    }

    pub fn main_loop(&self) {

    }
    fn str_to_buff(&mut self, s: &[u8]) {
        for c in s {
            self.wi = if self.wi == 4095 { 0 } else { self.wi + 1 };
            self.buf[self.wi] = *c;
        }
    }
}
