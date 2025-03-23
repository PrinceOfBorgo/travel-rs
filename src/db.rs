use crate::settings::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use surrealdb::{
    Surreal,
    engine::any::{Any, connect},
    opt::auth::Root,
};
use tokio::sync::OnceCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Count {
    count: i64,
}

impl Deref for Count {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.count
    }
}

/// Panics if couldn't retrieve the db info from config file.
async fn connect_to_db() -> Result<Surreal<Any>, surrealdb::Error> {
    let Database {
        address,
        username,
        password,
        namespace,
        database,
    } = &SETTINGS.database;

    tracing::info!("Connecting to database {address}::{namespace}::{database}...");

    let db = connect(address).await?;

    db.signin(Root { username, password }).await?;
    db.use_ns(namespace).use_db(database).await?;

    tracing::info!("Connected to database {address}::{namespace}::{database}");

    Ok(db)
}

/// Panics if couldn't connect to database.
pub async fn db() -> &'static Surreal<Any> {
    static DB: OnceCell<Surreal<Any>> = OnceCell::const_new();
    DB.get_or_init(async || connect_to_db().await.unwrap())
        .await
}
