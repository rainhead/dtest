use diesel::prelude::*;
use diesel::dsl::*;
use diesel::sqlite::SqliteConnection;
use serde_json::to_string;

use dtest::models::*;
use dtest::schema::*;
use diesel::insert_into;

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("dtest.sqlite")
        .expect("Couldn't open database file.")
}

pub fn main() -> QueryResult<()> {
    let conn = establish_connection();
    Peer::create_local_peer(&conn);

    SendMessageEvent::create_local(&conn, String::from("Hello, world."));

    let peer2_id = Peer::create(&conn);

    IIdentifyWithEvent::create_local(&conn, peer2_id);

    insert_into(time::table)
        .values(&(
            time::peer_id.eq(peer2_id),
            time::event_type.eq(to_string(&EventType::IIdentifyWithEvent).unwrap()),
            time::wall.eq(now),
            time::seq_no.eq(0))
        )
        .execute(&conn)?;
    let event_id = time::table.select(time::id).order(time::id.desc()).first(&conn)?;
    insert_into(i_identify_with_event::table)
        .values(IIdentifyWithEvent { asserted_at: event_id, with_id: Peer::local_peer_id(&conn) })
        .execute(&conn)?;

    insert_into(time::table)
        .values(&(
            time::peer_id.eq(peer2_id),
            time::event_type.eq(to_string(&EventType::MyNameIsEvent).unwrap()),
            time::wall.eq(now),
            time::seq_no.eq(1))
        )
        .execute(&conn)?;
    let event_id = time::table.select(time::id).order(time::id.desc()).first(&conn)?;
    insert_into(my_name_is_event::table)
        .values(MyNameIsEvent { asserted_at: event_id, name: String::from("Peter") })
        .execute(&conn)?;

    SendMessageEvent::run_rules(&conn);
    Message::run_rules(&conn);
    MessageBody::run_rules(&conn);
    MessageAuthor::run_rules(&conn);
//    IdentifyWithEvent::run_rules(&conn);
    MutuallyIdentify::run_rules(&conn);
    SamePerson::run_rules(&conn);
    PeerName::run_rules(&conn);

    let our_events = PortableEvents::peer_events_since(&conn, 1, -1);
    println!("{:?}", our_events);

    let their_events = PortableEvents::peer_events_since(&conn, 2, -1);
    println!("{:?}", their_events);

    Ok(())
}
