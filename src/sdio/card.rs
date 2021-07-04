
use crate::sdio::sdio_api::Rca;
use crate::sdio::sdio_api::SdioApi;

pub struct Card {
    api: SdioApi,
    rca: Rca
}

impl Card {
    pub(crate) fn new(api: SdioApi, rca: Rca) -> Self {
        Card {
            api: api,
            rca: rca
        }
    }
}

