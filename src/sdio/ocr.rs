pub struct Ocr {
    ocr: u32
}

impl Ocr {
    pub fn new(ocr: u32) -> Ocr {
        Ocr {
            ocr: ocr
        }
    }

    pub fn is_voltage(&self, voltage: u32) -> bool {
        (self.ocr & voltage) != 0
    }

    pub fn is_busy(&self) -> bool {
        (self.ocr & ((1 as u32) << 31)) != 0
    }

    pub fn is_ccs(&self) -> bool {
        (self.ocr & ((1 as u32) << 30)) != 0
    }
    pub fn is_uhs2(&self) -> bool {
        (self.ocr & ((1 as u32) << 29)) != 0
    }

    pub fn is_s18a(&self) -> bool {
        (self.ocr & ((1 as u32) << 24)) != 0
    }
}