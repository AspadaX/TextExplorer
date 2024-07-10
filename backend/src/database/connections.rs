use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use dotenvy::{dotenv, dotenv_override};
use std::env;


pub fn establish_connection() -> SqliteConnection {
    dotenv_override().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    return SqliteConnection::establish(&database_url)
        .unwrap_or_else(
            |_| panic!("Error connecting to {}", &database_url)
        );
}