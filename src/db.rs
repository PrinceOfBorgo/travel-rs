use {
    surrealdb::{
        engine::remote::ws::{Client, Ws},
        opt::auth::Root,
        Surreal,
    },
    tokio::sync::OnceCell,
};

use crate::CONFIG;

/// Panics if couldn't retrieve the db info from config file.
async fn connect_to_db() -> Result<Surreal<Client>, surrealdb::Error> {
    tracing::info!("Connecting to database...");

    let host = CONFIG.get::<String>("host").unwrap();
    let port = CONFIG.get::<usize>("port").unwrap();
    let username = &CONFIG.get::<String>("username").unwrap();
    let password = &CONFIG.get::<String>("password").unwrap();

    let db = Surreal::new::<Ws>(format!("{host}:{port}")).await?;
    db.signin(Root { username, password }).await?;
    db.use_ns("travel_rs").use_db("travel_rs_db").await?;

    tracing::info!("Connected to database.");

    Ok(db)
}

pub async fn init_db() -> Result<(), surrealdb::Error> {
    tracing::info!("Initializing database...");
    let db = db().await;

    db.query(
        "
DEFINE TABLE traveler SCHEMAFULL;
DEFINE FIELD chat_id ON TABLE traveler TYPE number;
DEFINE FIELD name ON TABLE traveler TYPE string ASSERT string::len($value) > 0 ;
DEFINE INDEX traveler_chat_id_name_index ON traveler FIELDS chat_id, name UNIQUE;
",
    )
    .await?;

    tracing::info!("Database initialized.");
    Ok(())
}

/// Panics if couldn't connect to database.
pub async fn db() -> &'static Surreal<Client> {
    static DB: OnceCell<Surreal<Client>> = OnceCell::const_new();
    DB.get_or_init(|| async { connect_to_db().await.unwrap() })
        .await
}
