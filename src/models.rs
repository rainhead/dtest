use crate::schema::*;

use chrono::prelude::*;
use diesel::prelude::*;
use diesel::dsl::*;
use diesel::sqlite::SqliteConnection;
use diesel::sql_query;

pub trait Relation {
    fn run_rules(conn: &SqliteConnection) -> usize;
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="entity"]
#[belongs_to(Event, foreign_key="introduced_at")]
pub struct Entity {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub introduced_at: i32,
}
impl Entity {
    pub fn create(conn: &SqliteConnection, event_id: i32) -> i32 {
        let message_uuid = uuid::Uuid::new_v4();
        insert_into(entity::table)
            .values(&(
                entity::uuid.eq(message_uuid.to_string()),
                entity::introduced_at.eq(event_id)
            ))
            .execute(conn)
            .unwrap();
        entity::table.select(entity::id).order(entity::id.desc()).first(conn).unwrap()
    }
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

    pub fn create(conn: &SqliteConnection) -> i32 {
        insert_into(peer::table)
            .default_values()
            .execute(conn)
            .unwrap();
        peer::table.select(peer::id).order(peer::id.desc()).first(conn).unwrap()
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
            .select(sql("seq_no + 1"))
            .first(conn)
            .unwrap_or_default()
    }

    pub fn create_local(conn: &SqliteConnection) -> i32 {
        let peer_id = Peer::local_peer_id(conn);
        let seq_no = Self::next_seq_no_for_peer(peer_id, conn);
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

// workaround for asserted_at + retracted_at per https://github.com/diesel-rs/diesel/issues/89
pub struct Retraction(pub Event);

#[derive(Identifiable, Queryable, Associations, Insertable, PartialEq, Debug)]
#[table_name="send_message_event"]
#[belongs_to(Event, foreign_key="asserted_at")]
#[primary_key(asserted_at)]
pub struct SendMessageEvent {
    pub asserted_at: i32,
    pub message_id: i32,
    pub body: String,
}
impl SendMessageEvent {
    pub fn create_local(conn: &SqliteConnection, body: String) {
        let event_id = Event::create_local(conn);
        let entity_id = Entity::create(conn, event_id);
        insert_into(send_message_event::table)
            .values(&(
                send_message_event::asserted_at.eq(event_id),
                send_message_event::message_id.eq(entity_id),
                send_message_event::body.eq(body)
            ))
            .execute(conn)
            .unwrap();
    }
}
impl Relation for SendMessageEvent {
    fn run_rules(conn: &SqliteConnection) -> usize {
        send_message_event::table
            .select((send_message_event::asserted_at,))
            .left_outer_join(entity::table.on(send_message_event::asserted_at.eq(entity::introduced_at)))
            .filter(entity::introduced_at.is_null())
            .insert_into(entity::table)
            .into_columns((entity::introduced_at,))
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
impl Relation for Message {
    fn run_rules(conn: &SqliteConnection) -> usize {
        entity::table
            .select((entity::id,))
            .inner_join(send_message_event::table.on(send_message_event::asserted_at.eq(entity::introduced_at)))
            .left_outer_join(message::table)
            .filter(message::entity_id.is_null())
            .insert_into(message::table)
            .into_columns((message::entity_id,))
            .execute(conn)
            .unwrap()
    }
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
impl Relation for MessageBody {
    fn run_rules(conn: &SqliteConnection) -> usize {
        send_message_event::table
            .inner_join(entity::table.on(send_message_event::asserted_at.eq(entity::introduced_at)))
            .left_outer_join(
                message_body::table.on(entity::id.eq(message_body::entity_id)
                    .and(entity::introduced_at.eq(message_body::asserted_at))))
            .filter(message_body::entity_id.is_null())
            .select((entity::id, send_message_event::asserted_at, send_message_event::body))
            .insert_into(message_body::table)
            .execute(conn)
            .unwrap()
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="message_author"]
#[primary_key(entity_id)]
#[belongs_to(Entity)]
#[belongs_to(Event, foreign_key="asserted_at")]
pub struct MessageAuthor {
    pub entity_id: i32,
    pub asserted_at: i32,
    pub author_id: i32,
}
impl Relation for MessageAuthor {
    fn run_rules(conn: &SqliteConnection) -> usize {
        send_message_event::table
            .inner_join(entity::table.on(send_message_event::asserted_at.eq(entity::introduced_at)))
            .inner_join(event::table)
            .left_outer_join(message_author::table.on(entity::id.eq(message_author::entity_id)))
            .filter(message_author::entity_id.is_null())
            .select((entity::id, send_message_event::asserted_at, event::peer_id))
            .insert_into(message_author::table)
            .execute(conn)
            .unwrap()
    }
}

#[derive(Identifiable, Insertable, Queryable, Associations, PartialEq, Debug)]
#[table_name="identify_with_event"]
#[primary_key(asserted_at)]
#[belongs_to(Peer, foreign_key="with_id")]
pub struct IdentifyWithEvent {
    pub asserted_at: i32,
    pub with_id: i32,
}
impl IdentifyWithEvent {
    pub fn create_local(conn: &SqliteConnection, with_id: i32) {
        let event_id = Event::create_local(conn);
        insert_into(identify_with_event::table)
            .values(&(identify_with_event::asserted_at.eq(event_id), identify_with_event::with_id.eq(with_id)))
            .execute(conn)
            .unwrap();
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="mutually_identify"]
#[primary_key(left_id, right_id)]
#[belongs_to(Peer, foreign_key="left_id")]
pub struct MutuallyIdentify {
    pub left_id: i32,
    pub right_id: i32
}
impl Relation for MutuallyIdentify {
    fn run_rules(conn: &SqliteConnection) -> usize {
        sql_query("
            INSERT INTO mutually_identify
            SELECT new.left_id, new.right_id FROM (
                SELECT id AS left_id, id AS right_id FROM peer
                UNION
                SELECT event1.peer_id AS left_id, event2.peer_id AS right_id
                FROM identify_with_event AS id1 JOIN event AS event1 ON id1.asserted_at = event1.id
                JOIN identify_with_event AS id2 ON event1.peer_id = id2.with_id
                JOIN event AS event2 ON id2.asserted_at = event2.id AND event2.peer_id = id1.with_id
            ) AS new
            LEFT JOIN mutually_identify AS old ON new.left_id = old.left_id AND new.right_id = old.right_id
            WHERE old.left_id IS NULL
        ").execute(conn).unwrap()
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="same_person"]
#[primary_key(left_id, right_id)]
#[belongs_to(Peer, foreign_key="left_id")]
pub struct SamePerson {
    pub left_id: i32,
    pub right_id: i32
}
impl Relation for SamePerson {
    fn run_rules(conn: &SqliteConnection) -> usize {
        sql_query("
            WITH RECURSIVE same AS (
                SELECT left_id, right_id FROM mutually_identify
                UNION
                SELECT same.left_id, mut.right_id
                FROM mutually_identify AS mut
                JOIN same ON same.right_id = mut.left_id
            )
            INSERT INTO same_person
            SELECT same.left_id, same.right_id
            FROM same
            LEFT JOIN same_person AS old ON same.left_id = old.left_id AND same.right_id = old.right_id
            WHERE old.left_id IS NULL
        ").execute(conn).unwrap()
    }
}

#[derive(Identifiable, Insertable, Queryable, Associations, PartialEq, Debug)]
#[table_name="peer_name_event"]
#[primary_key(asserted_at)]
#[belongs_to(Event, foreign_key="asserted_at")]
pub struct PeerNameEvent {
    pub asserted_at: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name="peer_name"]
#[primary_key(peer_id, asserted_at)]
#[belongs_to(Peer)]
#[belongs_to(Event, foreign_key="asserted_at")]
#[belongs_to(Retraction, foreign_key="retracted_at")]
pub struct PeerName {
    pub peer_id: i32,
    pub asserted_at: i32,
    pub retracted_at: Nullable<i32>,
    pub name: String,
}
impl Relation for PeerName {
    fn run_rules(conn: &SqliteConnection) -> usize {
        sql_query("
            INSERT INTO peer_name
            SELECT sp.right_id AS peer_id, event.id AS asserted_at, lag(event.id) OVER by_peer AS retracted_at, pne.name
            FROM peer_name_event AS pne
            JOIN event ON pne.asserted_at = event.id
            JOIN same_person AS sp ON sp.left_id = event.peer_id
            WINDOW by_peer AS (
                PARTITION BY sp.right_id ORDER BY event.ts DESC ROWS BETWEEN 1 PRECEDING AND CURRENT ROW
            )
        ").execute(conn).unwrap()
    }
}
