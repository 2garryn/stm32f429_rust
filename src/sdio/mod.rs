use stm32f4::stm32f429;

mod card_status;
pub mod card;
mod ocr;
mod cid;
mod csd;
mod sdio_api;
mod sdio_dma;

use card_status::CardStatusErr;
use sdio_api::Rca;
use sdio_api::SdioApi;
use card::Card;
use sdio_dma::SdioDma;
use crc::{Crc, CRC_32_ISO_HDLC};

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
#[derive(Debug)]
pub enum CardError {
    Timeout(u8),
    Ccrcfail(u8),
    StatusR1Err(CardStatusErr),
    OperationalSetError,
    UnknownError, 
    PowerDown,
    R6ComCrcError,
    R6IllegalCommand, 
    R6Error,
    InvalidBufferLen
}


pub fn new<'a>(sdio: stm32f429::SDIO, dma: &'a stm32f429::DMA2, gpiod: &mut stm32f429::GPIOD, gpioc: &mut stm32f429::GPIOC) -> Result<Card<'a>, CardError> {
    let tdma = SdioDma::new(dma);
    tdma.init();


    gpioc.moder.modify(|_r, w|
        w.moder8().alternate()
        .moder9().alternate()
        .moder10().alternate()
        .moder11().alternate()
        .moder12().alternate());
    gpiod.moder.modify(|_r, w| w.moder2().alternate());

    gpioc.afrh.modify(|_r, w|
        w.afrh8().af12()
        .afrh9().af12()
        .afrh10().af12()
        .afrh11().af12()
        .afrh12().af12());
    gpiod.afrl.modify(|_r, w| w.afrl2().af12());
     
    gpioc.ospeedr.modify(|_r, w|
        w.ospeedr8().very_high_speed()
        .ospeedr9().very_high_speed()
        .ospeedr10().very_high_speed()
        .ospeedr11().very_high_speed()
        .ospeedr12().very_high_speed());
    gpiod.ospeedr.modify(|_r, w| w.ospeedr2().very_high_speed());

    gpioc.otyper.modify(|_r, w|
        w.ot8().push_pull()
        .ot9().push_pull()
        .ot10().push_pull()
        .ot11().push_pull()
        .ot12().push_pull());
    gpiod.otyper.modify(|_r, w| w.ot2().push_pull());
 
    gpioc.pupdr.modify(|_r, w|
        w.pupdr8().floating()
        .pupdr9().floating()
        .pupdr10().floating()
        .pupdr11().floating()
        .pupdr12().floating());
    gpiod.pupdr.modify(|_r, w| w.pupdr2().pull_up());

    let sdio_api = SdioApi::new(sdio);
    sdio_api.preinit();
    sdio_api.clk_disabled();
    sdio_api.power_on();
    sdio_api.no_op(180000);
    sdio_api.clk_enabled();
    sdio_api.cmd0()?;
    sdio_api.cmd8()?;
    if let Some(e) = sdio_api.cmd55(&Rca::init_rca())?.any_error() {
        return Err(CardError::StatusR1Err(e));
    }
    sdio_api.acmd41()?;
    if !sdio_api.is_power_on() {
        return Err(CardError::PowerDown)
    }
    sdio_api.cmd2()?;
    let new_rca = sdio_api.cmd3()?;
    sdio_api.cmd9(&new_rca)?;
    if let Some(e) = sdio_api.cmd7(&new_rca)?.any_error() {
        return Err(CardError::StatusR1Err(e));
    }
    
    sdio_api.clk_disabled();
    sdio_api.increase_clockrate();
    sdio_api.clk_enabled();
    
    if let Some(e) = sdio_api.cmd55(&new_rca)?.any_error() {
        return Err(CardError::StatusR1Err(e));
    }
    sdio_api.acmd6()?;
    sdio_api.clk_disabled();
    sdio_api.set_wide_bus();
    sdio_api.clk_enabled();


    
    return Ok(Card::new(sdio_api, tdma, new_rca))
    
}

pub fn crc32(data: &[u8]) -> u32 {
    CRC32.checksum(data)
}