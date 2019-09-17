use crate::schema::*;

use chrono::prelude::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{insert_into, select};

pub trait Relation {
    fn run_rules(conn: &SqliteConnection) -> usize;
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="entity"]
#[belongs_to(Event, foreign_key="introduced_in")]
pub struct Entity {
    pub id: i32,
    pub introduced_in: i32,
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name="peer"]
// Ideally, peers would be entities
pub struct Peer {
    pub id: i32,
}
impl Peer {
    pub fn local_peer_id(conn: &SqliteConnection) -> i32 {
        peer::table.filter(peer::is_local).select(peer::id).first(conn).unwrap()
    }

    pub fn create_local_peer(conn: &SqliteConnection) {
        insert_into(peer::table)
            .values(&(
                peer::is_local.eq(true),
            ))
            .execute(conn)
            .expect("failed to create local peer. Maybe it already exists?");
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="event"]
#[belongs_to(Peer)]
pub struct Event {
    pub id: i32,
    pub ts: chrono::NaiveDateTime,
    pub peer_id: i32,
    pub seq_no: i32,
}
impl Event {
    pub fn next_seq_no_for_peer(peer_id: i32, conn: &SqliteConnection) -> i32 {
        event::table
            .filter(event::peer_id.eq(peer_id))
            .select(event::seq_no)
            .first::<i32>(conn)
            .unwrap_or_default()
            + 1
    }

    pub fn create_local(conn: &SqliteConnection) -> i32 {
        let peer_id = Peer::local_peer_id(conn);
        let seq_no = Self::next_seq_no_for_peer(peer_id, conn);
        let now = select(diesel::dsl::now).get_result::<NaiveDateTime>(conn).unwrap();
        insert_into(event::table)
            .values(&(
                event::ts.eq(now),
                event::peer_id.eq(peer_id),
                event::seq_no.eq(seq_no),
            ))
            .execute(conn)
            .unwrap();
        event::table.select(event::id).order(event::id.desc()).first(conn).unwrap()
    }
}

#[derive(Insertable)]
#[table_name="event"]
pub struct LocalEvent {
    pub ts: chrono::NaiveDateTime,
}

// workaround for asserted_at + retracted_at per https://github.com/diesel-rs/diesel/issues/89
pub struct Retraction(pub Event);

#[derive(Identifiable, Queryable, Associations, Insertable, PartialEq, Debug)]
#[table_name="send_message_event"]
#[belongs_to(Event, foreign_key="asserted_at")]
#[primary_key(asserted_at)]
pub struct SendMessageEvent {
    pub asserted_at: i32,
    pub body: String,
}
impl Relation for SendMessageEvent {
    fn run_rules(conn: &SqliteConnection) -> usize {
        send_message_event::table
            .select((send_message_event::asserted_at,))
            .left_outer_join(entity::table.on(send_message_event::asserted_at.eq(entity::introduced_in)))
            .filter(entity::introduced_in.is_null())
            .insert_into(entity::table)
            .into_columns((entity::introduced_in,))
            .execute(conn)
            .unwrap()
    }
}



#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="message"]
#[belongs_to(Entity)]
#[primary_key(entity_id)]
pub struct Message {
    pub entity_id: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="message_body"]
#[primary_key(entity_id, asserted_at)]
#[belongs_to(Entity)]
#[belongs_to(Event, foreign_key="asserted_at")]
pub struct MessageBody {
    pub entity_id: i32,
    pub asserted_at: i32,
    pub body: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="peer_name"]
#[primary_key(peer_id, asserted_at)]
#[belongs_to(Peer)]
#[belongs_to(Event, foreign_key="asserted_at")]
#[belongs_to(Retraction, foreign_key="retracted_at")]
pub struct PeerName {
    pub peer_id: i32,
    pub asserted_at: i32,
    pub retracted_at: i32,
    pub name: String,
}
