use stm32f4::stm32f429;
use crate::sdio::card_status::*;
use crate::sdio::ocr::*;
use crate::sdio::cid::*;
use crate::sdio::csd::*;
use crate::sdio::CardError;


pub enum WaitResp {
    NoResponse, ShortResponse,
    NoResponse2, LongResponse
}

pub struct Rca(u16);
impl Rca {
    pub fn init_rca() -> Rca {
        Rca(0)
    }
    pub fn as_cmd_arg(&self) -> u32 {
        (self.0 as u32) << 16
    }
}

pub struct Cmd {
    pub cmdindex: u8,
    pub waitresp: WaitResp,
    pub waitint: bool,
    pub waitpend: bool,
    pub cpsmen: bool,
    pub sdio_suspend: bool,
    pub encmdcompl: bool,
    pub nien: bool,
    pub atacmd: bool
}


pub struct SdioApi {
    sdio: stm32f429::SDIO
}

impl SdioApi {
    pub fn new(sdio: stm32f429::SDIO) -> Self {
        Self {
            sdio: sdio
        }
    }

    pub fn preinit(&self) {
        self.sdio.clkcr.modify(|_, w| 
            w.negedge().rising()
            .bypass().disabled()
            .pwrsav().enabled()
            .widbus().bus_width1()
            .hwfc_en().disabled()
            .clkdiv().bits(148) // 300khz
        );
    }

    pub fn clk_enabled(&self) { 
        self.sdio.clkcr.modify(|_, w| w.clken().enabled()) 
    }
    pub fn clk_disabled(&self) { 
        self.sdio.clkcr.modify(|_, w| w.clken().disabled()) 
    }
    pub fn power_on(&self) {
        self.sdio.power.modify(|_, w| w.pwrctrl().power_on());
    }
    pub fn is_power_on(&self) -> bool {
        self.sdio.power.read().pwrctrl().is_power_on()
    }
    pub fn set_wide_bus(&self) {
        self.sdio.clkcr.modify(|_r, w| w.widbus().bus_width4());
    }
    pub fn bypass_div(&self) {
        self.sdio.clkcr.modify(|_r, w| w.bypass().enabled().clken().enabled());
    }
    pub fn no_op(&self, clk: u64) {
        for _ in 0..clk {
            cortex_m::asm::nop();
        }
    }
    pub fn cmd0(&self) -> Result<(), CardError> {
        self.cmd_send_simple(0, 0, WaitResp::NoResponse);
        self.parse_no_response(0)
    }
    pub fn cmd2(&self) -> Result<Cid, CardError> {
        self.cmd_send_simple(2, 0, WaitResp::LongResponse);
        let (r1, r2, r3, r4) = self.parse_response2(2)?;
        Ok(Cid::new(r1, r2, r3, r4))
    }
    pub fn cmd3(&self) -> Result<Rca, CardError> {
        self.cmd_send_simple(3, 0, WaitResp::ShortResponse);
        let r1 = self.parse_response6(3)?;
        // General Unkown Error
        if (r1 & ((1 as u32) << 13)) != 0 { return Err(CardError::R6Error) }
        // Illegal command
        if (r1 & ((1 as u32) << 14)) != 0 { return Err(CardError::R6IllegalCommand)}
        // Com Crc Error
        if (r1 & ((1 as u32) << 13)) != 0 { return Err(CardError::R6ComCrcError) }
        Ok(Rca((r1 >> 16) as u16))

    }
    pub fn acmd6(&self) -> Result<CardStatus, CardError> {
        self.cmd_send_simple(6, 0b10, WaitResp::ShortResponse);
        let r = self.parse_response1(6)?;
        Ok(r)
    }

    pub fn cmd9(&self, rca: &Rca) -> Result<Csd, CardError> {
        self.cmd_send_simple(9, rca.as_cmd_arg(), WaitResp::LongResponse);
        let (r1, r2, r3, r4) = self.parse_response2(9)?;
        Ok(Csd::new(r1, r2, r3, r4))
    }

    pub fn cmd7(&self, rca: &Rca) -> Result<CardStatus, CardError> {
        self.cmd_send_simple(7, rca.as_cmd_arg(), WaitResp::ShortResponse);
        let r = self.parse_response1(7)?;
        Ok(r)
    }

    // SEND_IF_COND available only for card v2.0
    pub fn cmd8(&self) -> Result<(), CardError> {
        /*
            Voltage suplied 2.7 - 3.6v: 0001b
            Check Pattern: 10101010b
            PCIe 1.2v support - not asking: 0b
            PCIe availability - not asking: 0b
        */
        let args = 0x000001AA; // 000110101010b
        self.cmd_send_simple(8, args, WaitResp::ShortResponse);
        self.timeout(|| self.command_is_finished(), 5000, 8)?;
        self.check_error_flags(8)?;
        self.clear_static_flags();
        Ok(())
    }
    pub fn cmd55(&self, rca: &Rca) -> Result<CardStatus, CardError> {
        self.cmd_send_simple(55, rca.as_cmd_arg(), WaitResp::ShortResponse);
        self.parse_response1(55)
    }
    // SD_SEND_OP_COND
    pub fn acmd41(&self) -> Result<Ocr, CardError>{
        let voltage: u32 = 1 << 20; // voltage 3.2-3.3 (OCR)
        let sdhc: u32 = 1 << 30; //SDHC or SDXC (HCS Bit)
        let max_perf: u32 = 1 << 28; // XPC - maximum performance
        let busy: u32 = 1 << 31; //Busy 
        let args: u32 = voltage | sdhc | max_perf | busy;
        let init_rca = &Rca::init_rca();
        for _ in 1..0xFFFF {
            self.clear_static_flags();
            let sta = self.cmd55(init_rca)?;
            if let Some(e) = sta.any_error() {
                return Err(CardError::StatusR1Err(e));
            }
            self.cmd_send_simple(41, args, WaitResp::ShortResponse);
            let resp = self.parse_response3(41)?;
            if resp.is_busy() {
                return Ok(resp);
            } 
        }
        Err(CardError::OperationalSetError)
    }
    fn parse_no_response(&self, cmdn: u8) -> Result<(), CardError> {
        let f = || self.sdio.sta.read().cmdsent().is_sent();
        self.timeout(f, 5000, cmdn)?;
        self.clear_static_flags();
        Ok(()) 
    }
    fn parse_response1(&self, cmdn: u8) -> Result<CardStatus, CardError>{
        self.await_finished(cmdn)?;
        self.check_error_flags(cmdn)?;
        self.clear_static_flags();
        if self.sdio.respcmd.read() != (cmdn as u32) {
            return Err(CardError::Ccrcfail(cmdn));
        } 
        let r1 = self.sdio.resp1.read().cardstatus1().bits();
        Ok(CardStatus::new(r1))
    }
    fn parse_response2(&self, cmdn: u8) -> Result<(u32, u32, u32, u32), CardError> {
        self.await_finished(cmdn)?;
        self.check_error_flags(cmdn)?;
        self.clear_static_flags();
        let r1 = self.sdio.resp1.read().cardstatus1().bits();
        let r2 = self.sdio.resp2.read().cardstatus2().bits();
        let r3 = self.sdio.resp3.read().cardstatus3().bits();
        let r4 = self.sdio.resp4.read().cardstatus4().bits();
        Ok((r1, r2, r3, r4))
    }
    fn parse_response3(&self, cmdn: u8) -> Result<Ocr, CardError> {
        self.await_finished(cmdn)?;
        if self.sdio.sta.read().ctimeout().is_timeout() {
            self.sdio.icr.modify(|_, w| w.ctimeoutc().clear_bit());
            return Err(CardError::Timeout(cmdn));
        }
        self.clear_static_flags();
        let r1 = self.sdio.resp1.read().cardstatus1().bits();
        Ok(Ocr::new(r1))
    }
    fn parse_response6(&self, cmdn: u8) -> Result<u32, CardError> {
        self.await_finished(cmdn)?;
        self.check_error_flags(cmdn)?;
        self.clear_static_flags();
        let r1 = self.sdio.resp1.read().cardstatus1().bits();
        Ok(r1)
    }
    /* 
        HELPERS
    */
    fn clear_static_flags(&self) {
        self.sdio.icr.modify(|_, w| 
            w.ccrcfailc().set_bit()
            .ctimeoutc().set_bit()
            .cmdrendc().set_bit()
            .cmdsentc().set_bit()
        );
    }
    fn command_is_finished(&self) -> bool {
        let sta = self.sdio.sta.read();
        sta.ccrcfail().is_failed() || 
        sta.cmdrend().is_done() || 
        sta.ctimeout().is_timeout() || 
        sta.cmdact().is_not_in_progress()
    }
    fn check_error_flags(&self, cmdn: u8) -> Result<(), CardError> {
        self.no_op(200);
        if self.sdio.sta.read().ctimeout().is_timeout() {
            self.sdio.icr.modify(|_, w| w.ctimeoutc().clear_bit());
            return Err(CardError::Timeout(cmdn));
        }
        if self.sdio.sta.read().ccrcfail().is_failed() {
            self.sdio.icr.modify(|_, w| w.ccrcfailc().clear_bit());
            return Err(CardError::Ccrcfail(cmdn));
        }
        Ok(())
    }
    fn cmd_send_simple(&self, cmd: u8, args: u32, waitresp: WaitResp) {
        self.cmd_send(Cmd {
            cmdindex: cmd, 
            waitresp: waitresp,
            cpsmen: true,
            waitint: false,
            waitpend: false,
            sdio_suspend: false,
            encmdcompl: false, 
            nien: false,
            atacmd: false
        }, args)
    }
    fn cmd_send(&self, cmd: Cmd, args: u32) {
        while self.sdio.sta.read().cmdact().bit() {};
        self.sdio.arg.write(|w| w.cmdarg().bits(args));
        self.sdio.cmd.write(|w| {
            w.cmdindex().bits(cmd.cmdindex);
            match cmd.waitresp {
                WaitResp::NoResponse => w.waitresp().no_response(),
                WaitResp::ShortResponse => w.waitresp().short_response(),
                WaitResp::NoResponse2 => w.waitresp().no_response2(),
                WaitResp::LongResponse => w.waitresp().long_response()
            };
            if cmd.waitint { w.waitint().enabled(); }
            else { w.waitint().disabled();  }
            if cmd.waitpend { w.waitpend().enabled(); }
            if cmd.cpsmen { w.cpsmen().enabled(); }
            if cmd.sdio_suspend { w.sdiosuspend().enabled(); }
            if cmd.encmdcompl { w.encmdcompl().enabled(); }
            if cmd.nien { w.n_ien().enabled(); }
            if cmd.atacmd { w.ce_atacmd().enabled(); }
            w
        })
    }
    pub fn await_finished(&self, cmdn: u8) -> Result<(), CardError> {
        self.timeout(|| self.command_is_finished(), 5000, cmdn)
    }
    pub fn timeout<T: Fn() -> bool>(&self, f: T, ms: u16, cmdn: u8) -> Result<(), CardError> {
        //let  mut to: u64 = 1600000000 - (ms as u64);
        for _ in 0..16000000 {
            if f() { 
                return Ok(()); 
            }
        }
        return Err(CardError::Timeout(cmdn))
    }
}