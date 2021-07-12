
use crate::sdio::sdio_api::Rca;
use crate::sdio::sdio_api::SdioApi;
use crate::sdio::sdio_api::DataTransfMode;
use crate::sdio::CardError;
use crate::sdio::sdio_dma::SdioDma;

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

    pub fn read_block_dma(&self, buf: &mut [u8], block_addr: u32) -> Result<(), CardError> {
        self.wait_ready_for_data()?;
        self.dma.p2m(buf);
        self.api.dma_enable();
        if let Some(e) = self.api.cmd16()?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        self.api.preread_config(0xFFFFFFFF, self.api.default_block_size() * 1, DataTransfMode::Block);
        if let Some(e) = self.api.cmd17(block_addr)?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        Ok(())

    }

    pub fn read_block(&self, buf: &mut [u8], block_addr: u32) -> Result<(), CardError> {
        self.wait_ready_for_data()?;

        if let Some(e) = self.api.cmd16()?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        self.api.preread_config(0xFFFFFFFF, self.api.default_block_size() * 1, DataTransfMode::Block);
        if let Some(e) = self.api.cmd17(block_addr)?.any_error() {
            return Err(CardError::StatusR1Err(e));
        };
        
        let _ = self.api.read_block(buf);
        
        Ok(())
    }

    pub fn read_block_completed(&self)  {
        while !self.dma.p2m_completed() {};
        loop {
            if self.api.datacount() == 0 {
                break;
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

