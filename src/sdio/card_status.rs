
pub enum CurrentState {
    Idle, Ready, Ident, Stby, Tran,
    Data, Rcv, Prg, Dis
}

#[derive(Debug, Clone, Copy)]
pub enum CardStatusErr {
    OutOfRange = 31,
    AddressError = 30,
    BlockLenError = 29,
    EraseSeqError = 28,
    EraseParam = 27,
    WpViolation = 26,
    LockUnlockFailed = 24,
    ComCrcError = 23,
    IllegalCommand = 22,
    CardEccFailed = 21,
    CcError = 20,
    Error = 19,
    CsdOverride = 16,
    WpEraseSkip = 15,
    AkeSeqError = 3
}


pub enum CardStatusSts {
    CardIsLocked = 25,
    CardEccDisabled = 14,
    EraseReset = 13,
    ReadyForData = 8,
    FxEvent = 6,
    AppCmd = 5
}
pub struct CardStatus {
    status: u32
}

impl CardStatus {
    pub fn new(sts: u32) -> CardStatus {
        CardStatus {
            status: sts
        }
    }

    pub fn is_error(&self, err: CardStatusErr) -> bool {
        (self.status & (1 << (err as u8))) != 0
    }

    pub fn any_error(&self) -> Option<CardStatusErr> {
        match self.any_err_match_wrap() {
            Ok(_) => None,
            Err(e) => Some(e)
        }
    }

    fn any_err_match_wrap(&self) -> Result<(), CardStatusErr> {
        let f = |status: u32, err: CardStatusErr| -> Result<(), CardStatusErr> {
            if (status & (1 << (err as u8))) != 0 {
                return Err(err)
            }
            Ok(())
        };
        f(self.status, CardStatusErr::OutOfRange)?;
        f(self.status, CardStatusErr::AddressError)?;
        f(self.status, CardStatusErr::BlockLenError)?;
        f(self.status, CardStatusErr::EraseSeqError)?;
        f(self.status, CardStatusErr::EraseParam)?;
        f(self.status, CardStatusErr::WpViolation)?;
        f(self.status, CardStatusErr::LockUnlockFailed)?;
        f(self.status, CardStatusErr::ComCrcError)?;
        f(self.status, CardStatusErr::IllegalCommand)?;
        f(self.status, CardStatusErr::CardEccFailed)?;
        f(self.status, CardStatusErr::CcError)?;
        f(self.status, CardStatusErr::Error)?;
        f(self.status, CardStatusErr::CsdOverride)?;
        f(self.status, CardStatusErr::WpEraseSkip)?;
        f(self.status, CardStatusErr::AkeSeqError)?;
        Ok(())
    }

    pub fn is_flag(status: u32, sts: CardStatusSts) -> bool {
        (status & (1 << (sts as u8))) != 0
    }
}
