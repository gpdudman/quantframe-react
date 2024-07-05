use crate::{
    app::client::AppState, cache::client::CacheClient, log_parser::{enums::trade_classification::TradeClassification, types::create_stock_entity::CreateStockEntity}, logger, notification::client::NotifyClient, utils::modules::error::AppError
};

use entity::{enums::stock_type::StockType, stock::riven::attribute::RivenAttributeVec, sub_type::SubType, transaction::transaction::TransactionType};
use serde_json::json;
use service::{sea_orm::DatabaseConnection, StockItemMutation, StockItemQuery, StockRivenMutation, StockRivenQuery, TransactionMutation, TransactionQuery};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct DebugClient {
    log_file: String,
    app: Arc<Mutex<AppState>>,
    cache: Arc<Mutex<CacheClient>>,
    notify: Arc<Mutex<NotifyClient>>,
}

impl DebugClient {
    pub fn new(
        cache: Arc<Mutex<CacheClient>>,
        app: Arc<Mutex<AppState>>,
        notify: Arc<Mutex<NotifyClient>>,
    ) -> Self {
        DebugClient {
            log_file: "debug.log".to_string(),
            cache,
            app,
            notify,
        }
    }

    pub async fn migrate_data_transactions(
        &self,
        old_con: &DatabaseConnection,
        new_con: &DatabaseConnection,
    ) -> Result<(), AppError> {
        let cache = self.cache.lock()?.clone();
        let notify = self.notify.lock()?.clone();
        // Migrate the database transactions
        let old_items = TransactionQuery::get_old_transactions(old_con)
            .await
            .map_err(|e| AppError::new_db("MigrateDataBase", e))?;

        for item in old_items {
            
            let mut entity = CreateStockEntity::new(&item.url, item.price as i64);

            entity.sub_type = if item.rank > 0 || item.item_type == "riven" {
                Some(SubType {
                    rank: Some(item.rank as i64),
                    variant: None,
                    cyan_stars: None,
                    amber_stars: None,
                })
            } else {
                None
            };


            if item.item_type == "riven" {
                entity.entity_type = StockType::Riven;
                match item.properties {
                    Some(properties) => {
                        entity.mod_name = properties["mod_name"].as_str().unwrap_or("").to_string();
                        if entity.mod_name == "" {
                            entity.mod_name = properties["name"].as_str().unwrap_or("").to_string();
                        }
                        if entity.mod_name == "" {
                            entity.mod_name = "Unknown".to_string();
                        }
                        entity.mastery_rank = properties["mastery_level"].as_i64().unwrap_or(0);
                        entity.re_rolls = properties["re_rolls"].as_i64().unwrap_or(0);
                        entity.polarity = properties["polarity"].as_str().unwrap_or("").to_string();
                        match properties["attributes"].as_array() {
                        Some(attributes) => {
                            let mut new_attributes = vec![];
                            for attribute in attributes {
                                let attribute: entity::stock::riven::attribute::RivenAttribute =
                                    serde_json::from_value(attribute.clone()).unwrap();
                                new_attributes.push(attribute);
                            }
                            entity.attributes = new_attributes;
                        }
                        None => {}
                    };
                    }
                    None => {

                    }
                    
                }


            } else if item.item_type == "item" {
                entity.entity_type = StockType::Item;
            } 


            match entity.validate_entity(&cache, "--weapon_by url_name --weapon_lang en --item_by url_name --item_lang en --attribute_by url_name") {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            }

            let transaction_type =  match item.transaction_type.as_str() {
                "buy" => TransactionType::Purchase,
                "sell" => TransactionType::Sale,
                _ => {
                    return Err(AppError::new("MigrateDataBase", eyre::eyre!("Invalid transaction type")));
                }
            };

            let mut transaction = entity.to_transaction("", transaction_type)?;
            transaction.created_at = item.created.parse().unwrap();
            transaction.updated_at = item.created.parse().unwrap();
            match TransactionMutation::create_from_old(&new_con, transaction).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(AppError::new_db("MigrateDataBase", e));
                }                
            }
        }
        let new_items = TransactionQuery::get_all(new_con)
            .await
            .map_err(|e| AppError::new_db("MigrateDataBase", e))?;
        notify.gui().send_event_update(
            crate::utils::enums::ui_events::UIEvent::UpdateTransaction,
            crate::utils::enums::ui_events::UIOperationEvent::Set,
            Some(json!(new_items)),
        );
        Ok(())
    }

    pub async fn migrate_data_stock_item(
        &self,
        old_con: &DatabaseConnection,
        new_con: &DatabaseConnection,
    ) -> Result<(), AppError> {
        let cache = self.cache.lock()?.clone();
        let notify = self.notify.lock()?.clone();
        let old_items = StockItemQuery::get_old_stock_items(old_con)
            .await
            .map_err(|e| AppError::new_db("MigrateDataBase", e))?;
        for item in old_items {

            
            let mut entity = CreateStockEntity::new(&item.url, item.price as i64);
            entity.entity_type = StockType::Item;
            entity.sub_type = if item.rank > 0 {
                Some(SubType {
                    rank: Some(item.rank as i64),
                    variant: None,
                    cyan_stars: None,
                    amber_stars: None,
                })
            } else {
                None
            };

            match entity.validate_entity(&cache, "--item_by url_name --item_lang en") {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }

            let stock_item = entity.to_stock_item().to_stock();

            match StockItemMutation::create(&new_con, stock_item).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(AppError::new_db("MigrateDataBase", e));
                }                
            }
        }
        let new_items = StockItemQuery::get_all(new_con)
            .await
            .map_err(|e| AppError::new_db("MigrateDataBase", e))?;
        notify.gui().send_event_update(
            crate::utils::enums::ui_events::UIEvent::UpdateStockItems,
            crate::utils::enums::ui_events::UIOperationEvent::Set,
            Some(json!(new_items)),
        );
        Ok(())
    }

    pub async fn migrate_data_stock_riven(
        &self,
        old_con: &DatabaseConnection,
        new_con: &DatabaseConnection,
    ) -> Result<(), AppError> {
        let cache = self.cache.lock()?.clone();
        let notify = self.notify.lock()?.clone();
        let old_items = StockRivenQuery::get_old_stock_riven(old_con)
            .await
            .map_err(|e| AppError::new_db("MigrateDataBase", e))?;
        for item in old_items {
            let mut entity = CreateStockEntity::new(&item.weapon_url, item.price as i64);
            entity.entity_type = StockType::Riven;
            entity.mod_name = item.mod_name.clone();
            entity.mastery_rank = item.mastery_rank as i64;
            entity.re_rolls = item.re_rolls as i64;
            entity.polarity = item.polarity.clone();
            entity.attributes =item.attributes.clone().0;
            entity.sub_type = Some(SubType {
                    rank: Some(item.rank as i64),
                    variant: None,
                    cyan_stars: None,
                    amber_stars: None,
                });


            match entity.validate_entity(&cache, "--weapon_by url_name --weapon_lang en --attribute_by url_name") {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            }

            let stock_riven = entity.to_stock_riven().to_stock();
            match StockRivenMutation::create(&new_con, stock_riven).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(AppError::new_db("MigrateDataBase", e));
                }                                
            }
        }
        let new_items = StockRivenQuery::get_all(new_con)
            .await
            .map_err(|e| AppError::new_db("MigrateDataBase", e))?;
        notify.gui().send_event_update(
            crate::utils::enums::ui_events::UIEvent::UpdateStockRivens,
            crate::utils::enums::ui_events::UIOperationEvent::Set,
            Some(json!(new_items)),
        );
        Ok(())
    }
    pub async fn migrate_data_all(
        &self,
        old_con: &DatabaseConnection,
        new_con: &DatabaseConnection,
    ) -> Result<(), AppError> {
        self.migrate_data_transactions(old_con, new_con).await?;
        self.migrate_data_stock_item(old_con, new_con).await?;
        self.migrate_data_stock_riven(old_con, new_con).await?;
        Ok(())
    }
}
