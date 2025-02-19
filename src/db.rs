use crate::config::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
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
async fn connect_to_db() -> Result<Surreal<Client>, surrealdb::Error> {
    tracing::info!("Connecting to database...");

    let host = CONFIG.get::<String>(DB_HOST).unwrap();
    let port = CONFIG.get::<usize>(DB_PORT).unwrap();
    let username = &CONFIG.get::<String>(DB_USERNAME).unwrap();
    let password = &CONFIG.get::<String>(DB_PASSWORD).unwrap();
    let db_namespace = CONFIG.get::<String>(DB_NAMESPACE).unwrap();
    let db_database = CONFIG.get::<String>(DB_DATABASE).unwrap();

    let db = Surreal::new::<Ws>(format!("{host}:{port}")).await?;
    db.signin(Root { username, password }).await?;
    db.use_ns(db_namespace).use_db(db_database).await?;

    tracing::info!("Connected to database.");

    Ok(db)
}

/// Panics if couldn't connect to database.
pub async fn db() -> &'static Surreal<Client> {
    static DB: OnceCell<Surreal<Client>> = OnceCell::const_new();
    DB.get_or_init(|| async { connect_to_db().await.unwrap() })
        .await
}
