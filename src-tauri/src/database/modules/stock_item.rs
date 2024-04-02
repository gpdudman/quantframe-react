use crate::{
    database::{client::DBClient, enums::stock_status::StockStatus, types::price_history::PriceHistory}, helper, utils::{enums::log_level::LogLevel, modules::error::AppError}, wfm_client::types::order::Order
};
use eyre::eyre;

use sea_query::{ColumnDef, Expr, Iden, InsertStatement, Query, SqliteQueryBuilder, Table, Value};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Iden)]
pub enum StockItem {
    Table,
    Id,
    WFMId,
    Url,
    Name,
    Tags,
    Rank,
    SubType,
    Price,
    MiniumPrice,
    ListedPrice,
    PriceHistory,
    Owned,
    Hidden,
    Status,
    Created,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct StockItemStruct {
    pub id: i64,
    pub wfm_id: String,
    pub url: String,
    pub name: String,
    pub tags: String,
    pub rank: i32,
    pub sub_type: Option<String>,
    pub price: f64,
    pub minium_price: Option<i32>,
    pub listed_price: Option<i32>,
    pub price_history: sqlx::types::Json<Vec<PriceHistory>>,
    pub owned: i32,
    pub hidden: bool,
    pub status: String,
    pub created: String,
}

pub struct StockItemModule<'a> {
    pub client: &'a DBClient,
}

impl<'a> StockItemModule<'a> {
    // Methods sea-query

    // Initialize the database
    pub async fn initialize(&self) -> Result<bool, AppError> {
        let connection = self.client.connection.lock().unwrap().clone();
        let sql = Table::create()
            .table(StockItem::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(StockItem::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(StockItem::WFMId).uuid().not_null())
            .col(ColumnDef::new(StockItem::Url).string().not_null())
            .col(ColumnDef::new(StockItem::Name).string().not_null())
            .col(ColumnDef::new(StockItem::Tags).string().not_null())
            .col(
                ColumnDef::new(StockItem::Rank)
                    .integer()
                    .not_null()
                    .default(Value::Int(Some(0))),
            )
            .col(ColumnDef::new(StockItem::SubType).string())
            .col(
                ColumnDef::new(StockItem::Price)
                    .float()
                    .not_null()
                    .default(Value::Int(Some(0))),
            )
            .col(
                ColumnDef::new(StockItem::MiniumPrice)
                    .integer()
                    .default(Value::Int(None)),
            )
            .col(
                ColumnDef::new(StockItem::ListedPrice)
                    .integer()
                    .default(Value::Int(None)),
            )
            .col(
                ColumnDef::new(StockItem::PriceHistory)
                    .json()
                    .not_null()
                    .default(json!([])),
            )
            .col(
                ColumnDef::new(StockItem::Owned)
                    .integer()
                    .not_null()
                    .default(Value::Int(Some(1))),
            )
            .col(
                ColumnDef::new(StockItem::Hidden)
                    .boolean()
                    .not_null()
                    .default(Value::Bool(Some(false))),
            )
            .col(ColumnDef::new(StockItem::Created).date_time().not_null())
            .build(SqliteQueryBuilder);

        sqlx::query(&sql)
            .execute(&connection)
            .await
            .map_err(|e| AppError::new("Database", eyre!(e.to_string())))?;

        let mut table = Table::alter()
            .table(StockItem::Table)
            .add_column(
                ColumnDef::new(StockItem::MiniumPrice)
                    .integer()
                    .default(Value::Int(None)),
            )
            .to_string(SqliteQueryBuilder);
        helper::alter_table(connection.clone(), &table).await?;

        table = Table::alter()
            .table(StockItem::Table)
            .add_column(
                ColumnDef::new(StockItem::Hidden)
                    .boolean()
                    .not_null()
                    .default(Value::Bool(Some(false))),
            )
            .to_string(SqliteQueryBuilder);

        helper::alter_table(connection.clone(), &table).await?;

        table = Table::alter()
            .table(StockItem::Table)
            .add_column(
                ColumnDef::new(StockItem::Status)
                    .string()
                    .not_null()
                    .default(StockStatus::Pending.as_str()),
            )
            .to_string(SqliteQueryBuilder);
        helper::alter_table(connection.clone(), &table).await?;

        table = Table::alter()
            .table(StockItem::Table)
            .add_column(
                ColumnDef::new(StockItem::PriceHistory)
                    .json()
                    .not_null()
                    .default(json!([])),
            )
            .to_string(SqliteQueryBuilder);
        helper::alter_table(connection.clone(), &table).await?;

        Ok(true)
    }

    pub async fn get_items(&self) -> Result<Vec<StockItemStruct>, AppError> {
        let connection = self.client.connection.lock().unwrap().clone();
        // Read
        let sql = Query::select()
            .columns([
                StockItem::Id,
                StockItem::WFMId,
                StockItem::Url,
                StockItem::Name,
                StockItem::Tags,
                StockItem::Rank,
                StockItem::SubType,
                StockItem::Price,
                StockItem::MiniumPrice,
                StockItem::ListedPrice,
                StockItem::PriceHistory,
                StockItem::Owned,
                StockItem::Hidden,
                StockItem::Status,
                StockItem::Created,
            ])
            .from(StockItem::Table)
            .to_string(SqliteQueryBuilder);

        let rows = sqlx::query_as::<_, StockItemStruct>(&sql)
            .fetch_all(&connection)
            .await
            .unwrap();
        Ok(rows)
    }

    pub async fn get_item_by_url_name(
        &self,
        url_name: &str,
    ) -> Result<Option<StockItemStruct>, AppError> {
        let items = self.get_items().await?;
        let item = items.iter().find(|t| t.url == url_name);
        Ok(item.cloned())
    }
    pub async fn get_by_id(&self, id: i64) -> Result<Option<StockItemStruct>, AppError> {
        let stock = self.get_items().await?;
        let stock_item = stock.iter().find(|t| t.id == id);
        Ok(stock_item.cloned())
    }
    pub async fn create(
        &self,
        url_name: &str,
        mut quantity: i32,
        price: f64,
        minium_price: Option<i32>,
        rank: i32,
        sub_type: Option<&str>,
    ) -> Result<StockItemStruct, AppError> {
        let stock_items = self.get_item_by_url_name(url_name).await?;
        let connection = self.client.connection.lock().unwrap().clone();

        if quantity <= 0 {
            quantity = 1;
        }

        let item = self.client.cache.lock()?.item().find_type(&url_name)?;

        let item = match item {
            Some(t) => t,
            None => {
                return Err(AppError::new_with_level(
                    "Database",
                    eyre!("Item {} not found in cache", url_name),
                    LogLevel::Critical,
                ));
            }
        };

        let inventory = match stock_items {
            Some(t) => {
                let total_owned = t.owned + quantity;
                // Get price per unit
                let total_price = (t.price * t.owned as f64) + price as f64;
                let weighted_price = total_price / total_owned as f64;

                self.update_by_id(
                    t.id,
                    Some(total_owned),
                    Some(weighted_price),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
                let mut t = t.clone();
                t.owned = total_owned;
                t.price = weighted_price;
                t
            }
            None => {
                let price = price / (quantity as f64);

                let mut inventory = StockItemStruct {
                    id: 0,
                    wfm_id: item.clone().id,
                    url: item.clone().url_name,
                    name: item.clone().item_name,
                    tags: item.clone().tags.map(|t| t.join(",")).unwrap_or_default(),
                    rank: rank as i32,
                    sub_type: sub_type.map(|t| t.to_string()),
                    price: price as f64,
                    minium_price,
                    listed_price: None,
                    price_history: sqlx::types::Json(vec![]),
                    owned: quantity as i32,
                    hidden: false,
                    status: StockStatus::Pending.to_string(),
                    created: chrono::Local::now().naive_local().to_string(),
                };

                let sql = InsertStatement::default()
                    .into_table(StockItem::Table)
                    .columns([
                        StockItem::WFMId,
                        StockItem::Url,
                        StockItem::Name,
                        StockItem::Tags,
                        StockItem::Rank,
                        StockItem::SubType,
                        StockItem::Price,
                        StockItem::MiniumPrice,
                        StockItem::Owned,
                        StockItem::Hidden,
                        StockItem::Status,
                        StockItem::Created,
                    ])
                    .values_panic([
                        inventory.wfm_id.clone().into(),
                        inventory.url.clone().into(),
                        inventory.name.clone().replace("\'", "").into(),
                        inventory.tags.clone().into(),
                        inventory.rank.into(),
                        inventory.sub_type.clone().into(),
                        inventory.price.into(),
                        inventory.minium_price.into(),
                        inventory.owned.into(),
                        inventory.hidden.into(),
                        inventory.status.clone().into(),
                        inventory.created.clone().into(),
                    ])
                    .to_string(SqliteQueryBuilder);
                let row = sqlx::query(&sql)
                    .execute(&connection)
                    .await
                    .map_err(|e| AppError::new("Database", eyre!(e.to_string())))?;
                let id = row.last_insert_rowid();
                inventory.id = id;
                inventory
            }
        };
        // Update UI
        self.emit(
            "CREATE_OR_UPDATE",
            serde_json::to_value(inventory.clone()).unwrap(),
        );
        Ok(inventory)
    }

    pub async fn update_by_id(
        &self,
        id: i64,
        owned: Option<i32>,
        price: Option<f64>,
        minium_price: Option<i32>,
        listed_price: Option<i32>,
        status: Option<StockStatus>,
        hidden: Option<bool>,
        price_history: Option<PriceHistory>,
        trades: Option<&Vec<Order>>,
    ) -> Result<StockItemStruct, AppError> {
        let connection = self.client.connection.lock().unwrap().clone();

        let items = self.get_items().await?;
        let inventory = items.iter().find(|t| t.id == id);
        if inventory.is_none() {
            return Err(AppError::new_with_level(
                "Database",
                eyre!("Item not found in database"),
                LogLevel::Error,
            ));
        }
        let mut inventory = inventory.unwrap().clone();
        let mut values = vec![];

        if owned.is_some() {
            inventory.owned = owned.unwrap();
            values.push((StockItem::Owned, owned.into()));
        }

        if price.is_some() {
            inventory.price = price.unwrap();
            values.push((StockItem::Price, price.into()));
        }

        if minium_price.is_some() {
            // If minium_price is -1, set it to None
            let minium_price = if minium_price.unwrap() == -1 {
                None
            } else {
                minium_price
            };
            inventory.minium_price = minium_price;
            values.push((StockItem::MiniumPrice, minium_price.into()));
        }

        if listed_price.is_some() {
            let listed_price = if listed_price.unwrap() == -1 {
                None
            } else {
                listed_price
            };
            inventory.listed_price = listed_price;
            values.push((StockItem::ListedPrice, listed_price.into()));
        }

        if status.is_some() {
            inventory.status = status.unwrap().to_string();
            values.push((StockItem::Status, inventory.status.clone().into()));
        }

        if hidden.is_some() {
            inventory.hidden = hidden.unwrap();
            values.push((StockItem::Hidden, hidden.into()));
        }

        if price_history.is_some() {
            // let mut price_history = price_history.unwrap();
            let mut price_histories = inventory.price_history.clone();
            // Max 5 price history
            if price_histories.len() >= 5 {
                price_histories.remove(0);
            }
            price_histories.push(price_history.unwrap());
            inventory.price_history = price_histories;
            values.push((
                StockItem::PriceHistory,
                serde_json::to_value(sqlx::types::Json(&inventory.price_history.clone()))
                    .unwrap()
                    .into(),
            ));
        }

        let sql = Query::update()
            .table(StockItem::Table)
            .values(values)
            .and_where(Expr::col(StockItem::Id).eq(id))
            .to_string(SqliteQueryBuilder);
        sqlx::query(&sql.replace("\\", ""))
            .execute(&connection)
            .await
            .map_err(|e| AppError::new("Database", eyre!(e.to_string())))?;

        let mut json_data = serde_json::to_value(inventory.clone()).unwrap();
        json_data["trades"] = serde_json::to_value(trades).unwrap();
        self.emit("CREATE_OR_UPDATE", json_data);

        Ok(inventory.clone())
    }

    pub async fn reset_listed_price(&self) -> Result<(), AppError> {
        let connection = self.client.connection.lock().unwrap().clone();
        let sql = Query::update()
            .table(StockItem::Table)
            .values([
                (StockItem::ListedPrice, Value::Int(None)),
                (StockItem::Status, StockStatus::Pending.as_str().into()),
            ])
            .to_string(SqliteQueryBuilder);
        sqlx::query(&sql.replace("\\", ""))
            .execute(&connection)
            .await
            .map_err(|e| AppError::new("Database", eyre!(e.to_string())))?;

        self.emit("SET", json!(self.get_items().await?));
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<StockItemStruct, AppError> {
        let connection = self.client.connection.lock().unwrap().clone();
        let items = self.get_items().await?;

        let stock_item = items.iter().find(|t| t.id == id);
        if stock_item.is_none() {
            return Err(AppError::new_with_level(
                "Database",
                eyre!("Stock Item not found in database"),
                LogLevel::Error,
            ));
        }
        let sql = Query::delete()
            .from_table(StockItem::Table)
            .and_where(Expr::col(StockItem::Id).eq(id))
            .to_string(SqliteQueryBuilder);
        sqlx::query(&sql)
            .execute(&connection)
            .await
            .map_err(|e| AppError::new("Database", eyre!(e.to_string())))?;
        self.emit(
            "DELETE",
            serde_json::to_value(stock_item.unwrap().clone()).unwrap(),
        );
        Ok(stock_item.unwrap().clone())
    }

    pub async fn sell_item(&self, id: i64, mut quantity: i32) -> Result<StockItemStruct, AppError> {
        let items = self.get_items().await?;
        let stock_item = items.iter().find(|t| t.id == id);

        if stock_item.is_none() {
            return Err(AppError::new_with_level(
                "Database",
                eyre!("Item not found in database"),
                LogLevel::Error,
            ));
        }

        let mut inventory = stock_item.unwrap().clone();
        if quantity <= 0 {
            quantity = 1;
        }
        inventory.owned -= quantity;

        if inventory.owned <= 0 {
            self.delete(id).await?;
        } else {
            self.update_by_id(
                id,
                Some(inventory.owned.clone()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await?;
        }
        Ok(inventory.clone())
    }

    pub async fn get_items_names(&self) -> Result<Vec<String>, AppError> {
        let stock_items = self.get_items().await?;
        // Return all hidden items and where owned is under 1
        let stock_items = stock_items
            .iter()
            .filter(|t| t.hidden == false && t.owned > 0)
            .collect::<Vec<_>>();
        let names = stock_items
            .iter()
            .map(|t| t.url.clone())
            .collect::<Vec<_>>();
        Ok(names)
    }

    pub fn emit(&self, operation: &str, data: serde_json::Value) {
        helper::emit_update("StockItems", operation, Some(data));
    }
}
