pub mod schema;
pub mod models;

#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("dtest.sqlite")
        .expect("Couldn't open database file.")
}
