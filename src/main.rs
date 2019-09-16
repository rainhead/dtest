use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use dtest::models::*;

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("dtest.sqlite")
        .expect("Couldn't open database file.")
}

pub fn main() {
    let conn = establish_connection();
    Peer::create_local_peer(&conn);
    Event::create_local(&conn);
}
