use crate::{
    utils::{enums::log_level::LogLevel, modules::error::{ApiResult, AppError}}, wfm_client::{client::WFMClient, types::item::Item}
};

use eyre::eyre;
#[derive(Clone, Debug)]
pub struct ItemModule {
    pub client: WFMClient,
    pub debug_id: String,
    component: String,
}

impl ItemModule {
    pub fn new(client: WFMClient) -> Self {
        ItemModule {
            client,
            debug_id: "wfm_client_item".to_string(),
            component: "Items".to_string(),
        }
    }
    fn get_component(&self, component: &str) -> String {
        format!("{}:{}", self.component, component)
    }
    pub async fn get_all_items(&self) -> Result<Vec<Item>, AppError> {
        match self.client.get::<Vec<Item>>("items", Some("items")).await {
            Ok(ApiResult::Success(payload, _headers)) => {
                self.client.debug(
                    &self.debug_id,
                    &self.get_component("GetAllItems"),
                    format!("{} items were fetched.", payload.len()).as_str(),
                    None,
                );
                return Ok(payload);
            }
            Ok(ApiResult::Error(error, _headers)) => {
                return Err(self.client.create_api_error(
                    "Item:GetAllItems",
                    error,
                    eyre!("There was an error fetching items"),
                    LogLevel::Error,
                ));
            }
            Err(err) => {
                return Err(err);
            }
        };
    }
}
