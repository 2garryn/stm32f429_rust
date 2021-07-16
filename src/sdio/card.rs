
use crate::sdio::sdio_api::Rca;
use crate::sdio::sdio_api::SdioApi;
use crate::sdio::sdio_api::DataTransfMode;
use crate::sdio::CardError;
use crate::sdio::sdio_dma::SdioDma;

/*
    number of sectors = 31 116 288 (beginning from zero) 
*/

pub struct Card<'a> {
    api: SdioApi,
    rca: Rca,
    dma: SdioDma<'a>
}

impl<'a> Card <'a> {
    pub(crate) fn new(api: SdioApi, dma: SdioDma<'a>, rca: Rca) -> Self {
        Card {
            api: api,
            rca: rca,
            dma: dma
        }
    }

    pub fn read_block_dma(&self, buf: &mut [u8], block_addr: u32) -> Result<AwaitData, CardError> {
        if (buf.len() as u32) != self.api.default_block_size() {
            return Err(CardError::InvalidBufferLen)
        }
        self.wait_ready_for_data()?;
        self.dma.p2m(buf);
        self.api.dma_enable();
        if let Some(e) = self.api.cmd16()?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        self.api.preread_config(0xFFFFFFFF, self.api.default_block_size(), DataTransfMode::Block);
        if let Some(e) = self.api.cmd17(block_addr)?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        Ok(AwaitData{ multi: false })

    }

    pub fn read_multi_block_dma(&self, buf: &mut [u8], block_addr: u32) -> Result<AwaitData, CardError> {
        if ((buf.len() as u32) % self.api.default_block_size()) != 0 {
            return Err(CardError::InvalidBufferLen)
        }

        self.wait_ready_for_data()?;
        self.dma.p2m(buf);
        self.api.dma_enable();
        if let Some(e) = self.api.cmd16()?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        self.api.preread_config(0xFFFFFFFF, buf.len() as u32, DataTransfMode::Block);
        if let Some(e) = self.api.cmd18(block_addr)?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        Ok(AwaitData{ multi: true })
    }


    pub fn await_finished(&self, ad: AwaitData) -> Result<(), CardError> {
        while !self.dma.p2m_completed() {};
        loop {
            if self.api.datacount() == 0 {
                self.api.dctrl_disabled();
                self.api.dma_disable();
                if ad.multi {
                        self.api.cmd12()?;
                        self.api.clear_data_flags();
                }
               return Ok(())
            }
        }
    }

    fn wait_ready_for_data(&self) -> Result<(), CardError> {
        loop {
            let status = self.api.cmd13(&self.rca)?;
            if let Some(e) = status.any_error() {
                return Err(CardError::StatusR1Err(e));
            }
            if status.ready_for_data() {
                return Ok(())
            }
        }
    }
}


pub struct AwaitData {
    multi: bool,

}

