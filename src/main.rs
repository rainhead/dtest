use diesel::prelude::*;
use diesel::dsl::*;
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

    let peer2_id = Peer::create(&conn);
    let event_id = Event::create_local(&conn);
    insert_into(identify_with_event::table)
        .values(IdentifyWithEvent { asserted_at: event_id, with_id: peer2_id })
        .execute(&conn)
        .unwrap();

    insert_into(event::table)
        .values(&(event::peer_id.eq(peer2_id), event::ts.eq(now), event::seq_no.eq(0)))
        .execute(&conn)
        .unwrap();
    let event_id = event::table.select(event::id).order(event::id.desc()).first(&conn).unwrap();
    insert_into(identify_with_event::table)
        .values(IdentifyWithEvent { asserted_at: event_id, with_id: Peer::local_peer_id(&conn) })
        .execute(&conn)
        .unwrap();

    insert_into(event::table)
        .values(&(event::peer_id.eq(peer2_id), event::ts.eq(now), event::seq_no.eq(1)))
        .execute(&conn)
        .unwrap();
    let event_id = event::table.select(event::id).order(event::id.desc()).first(&conn).unwrap();
    insert_into(peer_name_event::table)
        .values(PeerNameEvent { asserted_at: event_id, name: String::from("Peter") })
        .execute(&conn)
        .unwrap();

    SendMessageEvent::run_rules(&conn);
    Message::run_rules(&conn);
    MessageBody::run_rules(&conn);
    MessageAuthor::run_rules(&conn);
//    IdentifyWithEvent::run_rules(&conn);
    MutuallyIdentify::run_rules(&conn);
    SamePerson::run_rules(&conn);
    PeerName::run_rules(&conn);
}
