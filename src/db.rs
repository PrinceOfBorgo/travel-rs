use crate::settings::*;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc};
use surrealdb::{
    Surreal,
    engine::any::{Any, connect},
    opt::auth::Root,
};

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
async fn connect_to_db() -> surrealdb::Result<Surreal<Any>> {
    let Database {
        address,
        username,
        password,
        namespace,
        database,
    } = &SETTINGS.database;

    tracing::info!("Connecting to database {address}::{namespace}::{database}...");

    let db = connect(address).await?;

    db.use_ns(namespace).use_db(database).await?;

    // Initialize the database if it is in memory
    if address == "memory" || address == "mem://" {
        let schema = std::fs::read_to_string("database/build_travelers_db.surql").unwrap();
        db.query(schema).await?;
    } else {
        // Authenticate only if it's not an in-memory database
        db.signin(Root { username, password }).await?;
    }

    tracing::info!("Connected to database {address}::{namespace}::{database}");

    Ok(db)
}

/// Panics if couldn't connect to database.
pub async fn db() -> Arc<Surreal<Any>> {
    let db_instance = connect_to_db().await.unwrap();
    Arc::new(db_instance)
}

#[cfg(test)]
mod tests {
    use super::*;

    test! { test_connect_to_db,
        let db = db().await;
        let version = db.version().await;
        assert!(
            version.is_ok(),
            "Failed to connect to the database: {:?}",
            version.err()
        );
    }
}
