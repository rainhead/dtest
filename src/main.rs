use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use dtest::models::*;
use dtest::schema::*;
use diesel::insert_into;

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("dtest.sqlite")
        .expect("Couldn't open database file.")
}

pub fn main() {
    let conn = establish_connection();
    Peer::create_local_peer(&conn);
    let event_id = Event::create_local(&conn);
    let msg = SendMessageEvent { asserted_at: event_id, body: String::from("Hello, world.") };
    insert_into(send_message_event::table)
        .values(&msg)
        .execute(&conn)
        .unwrap();
    SendMessageEvent::run_rules(&conn);
    Message::run_rules(&conn);
    MessageBody::run_rules(&conn);
}
