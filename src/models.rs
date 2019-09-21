use crate::schema::*;

use chrono::prelude::*;
use diesel::prelude::*;
use diesel::dsl::*;
use diesel::sqlite::SqliteConnection;
use diesel::sql_query;
use serde::{Serialize, Deserialize};
use serde_json::to_string;
use uuid::Uuid;

pub trait Relation {
    fn run_rules(conn: &SqliteConnection) -> usize;
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="entity"]
#[belongs_to(Time, foreign_key="introduced_at")]
pub struct Entity {
    pub id: i32,
    pub uuid: Uuid,
    pub introduced_at: i32,
}
impl Entity {
    pub fn create(conn: &SqliteConnection, event_id: i32) -> i32 {
        let message_uuid = Uuid::new_v4();
        insert_into(entity::table)
            .values(&(
                entity::uuid.eq(message_uuid.to_string()),
                entity::introduced_at.eq(event_id)
            ))
            .execute(conn)
            .unwrap();
        entity::table.select(entity::id).order(entity::id.desc()).first(conn).unwrap()
    }

    pub fn import(conn: &SqliteConnection, event_id: i32, uuid: Uuid) -> i32 {
        let existing_id = entity::table
            .select(entity::id)
            .filter(entity::uuid.eq(uuid.to_string()))
            .first(conn)
            .optional()
            .unwrap();
        match existing_id {
            Some(id) => id,
            None => {
                insert_into(entity::table)
                    .values(&(
                        entity::uuid.eq(uuid.to_string()),
                        entity::introduced_at.eq(event_id)
                    ))
                    .execute(conn)
                    .unwrap();
                Self::import(conn, event_id, uuid)
            }
        }
    }

    pub fn find_by_uuid(conn: &SqliteConnection, uuid: Uuid) -> i32 {
        entity::table
            .select(entity::id)
            .filter(entity::uuid.eq(uuid.to_string()))
            .first(conn)
            .expect(&format!("couldn't find entity with uuid {:?}", uuid))
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
                peer::uuid.eq(Uuid::new_v4().to_string()),
                peer::is_local.eq(true),
            ))
            .execute(conn)
            .expect("failed to create local peer. Maybe it already exists?");
    }

    pub fn create(conn: &SqliteConnection) -> i32 {
        insert_into(peer::table)
            .values(peer::uuid.eq(Uuid::new_v4().to_string()))
            .execute(conn)
            .unwrap();
        peer::table.select(peer::id).order(peer::id.desc()).first(conn).unwrap()
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="time"]
#[belongs_to(Peer)]
pub struct Time {
    pub id: i32,
    pub wall: chrono::NaiveDateTime,
    pub peer_id: i32,
    pub seq_no: i32,
    pub event_type: String,
}
impl Time {
    pub fn next_seq_no_for_peer(peer_id: i32, conn: &SqliteConnection) -> i32 {
        time::table
            .filter(time::peer_id.eq(peer_id))
            .select(sql("seq_no + 1"))
            .first(conn)
            .unwrap_or_default()
    }

    pub fn create_local<E: Event>(conn: &SqliteConnection) -> i32 {
        let peer_id = Peer::local_peer_id(conn);
        let seq_no = Self::next_seq_no_for_peer(peer_id, conn);
        insert_into(time::table)
            .values(&(
                time::wall.eq(now),
                time::event_type.eq(to_string(&E::EVENT_TYPE).unwrap()),
                time::peer_id.eq(peer_id),
                time::seq_no.eq(seq_no),
            ))
            .execute(conn)
            .unwrap();
        time::table.select(time::id).order(time::id.desc()).first(conn).unwrap()
    }
}

#[derive(Debug)]
pub struct PortableEvents {
    pub first_seq_no: i32,
    pub events: Vec<PortableEvent>,
}
impl PortableEvents {
    pub fn peer_events_since(conn: &SqliteConnection, peer_id: i32, since_seq_no: i32) -> Option<Self> {
        let events_in: Vec<(i32, chrono::NaiveDateTime, String)> = time::table
            .select((time::id, time::wall, time::event_type))
            .filter(time::peer_id.eq(peer_id))
            .filter(time::seq_no.gt(since_seq_no))
            .load(conn)
            .unwrap();
        if events_in.is_empty() { return None; }
        let events_out = events_in
            .into_iter()
            .map(|(time, wall, event_type)| {
                let event_type: EventType = serde_json::from_str(&event_type).unwrap();
                PortableEvent::fetch(conn, time, wall, event_type)
            })
            .collect();
        Some(PortableEvents {
            first_seq_no: since_seq_no + 1,
            events: events_out,
        })
    }
}

#[derive(Debug)]
pub struct PortableEvent {
    wall: chrono::NaiveDateTime,
    args: EventArguments,
}
impl PortableEvent {
    fn fetch(conn: &SqliteConnection, time: i32, wall: chrono::NaiveDateTime, event_type: EventType) -> Self {
        Self {
            wall,
            args: EventArguments::fetch(conn, time, event_type),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
pub enum EventType {
    SendMessageEvent,
    IIdentifyWithEvent,
    MyNameIsEvent,
}

#[derive(Debug)]
pub enum EventArguments {
    SendMessageEvent(<SendMessageEvent as Event>::Arguments),
    IIdentifyWithEvent(<IIdentifyWithEvent as Event>::Arguments),
    MyNameIsEvent(<MyNameIsEvent as Event>::Arguments),
}
impl EventArguments {
    fn fetch(conn: &SqliteConnection, time: i32, event_type: EventType) -> Self {
        match event_type {
            EventType::SendMessageEvent => Self::SendMessageEvent(SendMessageEvent::get_arguments(conn, time)),
            EventType::IIdentifyWithEvent => Self::IIdentifyWithEvent(IIdentifyWithEvent::get_arguments(conn, time)),
            EventType::MyNameIsEvent => Self::MyNameIsEvent(MyNameIsEvent::get_arguments(conn, time)),
        }
    }
}

// workaround for asserted_at + retracted_at per https://github.com/diesel-rs/diesel/issues/89
pub struct Retraction(pub Time);

pub trait Event {
    type Arguments;
    const EVENT_TYPE: EventType;

    fn get_arguments(conn: &SqliteConnection, time: i32) -> Self::Arguments;
}

#[derive(Identifiable, Queryable, Associations, Insertable, PartialEq, Debug)]
#[table_name="send_message_event"]
#[belongs_to(Time, foreign_key="asserted_at")]
#[belongs_to(Entity, foreign_key="message_id")]
#[primary_key(asserted_at)]
pub struct SendMessageEvent {
    pub asserted_at: i32,
    pub message_id: i32,
    pub body: String,
}
impl SendMessageEvent {
    pub fn create_local(conn: &SqliteConnection, body: String) {
        let event_id = Time::create_local::<Self>(conn);
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
impl Event for SendMessageEvent {
    type Arguments = (Uuid, String);
    const EVENT_TYPE: EventType = EventType::SendMessageEvent;

    fn get_arguments(conn: &SqliteConnection, time: i32) -> Self::Arguments {
        let record: (String, String) = send_message_event::table
            .inner_join(entity::table)
            .select((entity::uuid, send_message_event::body))
            .filter(send_message_event::asserted_at.eq(time))
            .first(conn)
            .unwrap();
        (Uuid::parse_str(&record.0).unwrap(), record.1)
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
#[belongs_to(Time, foreign_key="asserted_at")]
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
#[belongs_to(Time, foreign_key="asserted_at")]
pub struct MessageAuthor {
    pub entity_id: i32,
    pub asserted_at: i32,
    pub author_id: i32,
}
impl Relation for MessageAuthor {
    fn run_rules(conn: &SqliteConnection) -> usize {
        send_message_event::table
            .inner_join(entity::table.on(send_message_event::asserted_at.eq(entity::introduced_at)))
            .inner_join(time::table)
            .left_outer_join(message_author::table.on(entity::id.eq(message_author::entity_id)))
            .filter(message_author::entity_id.is_null())
            .select((entity::id, send_message_event::asserted_at, time::peer_id))
            .insert_into(message_author::table)
            .execute(conn)
            .unwrap()
    }
}

#[derive(Identifiable, Insertable, Queryable, Associations, PartialEq, Debug)]
#[table_name="i_identify_with_event"]
#[primary_key(asserted_at)]
#[belongs_to(Peer, foreign_key="with_id")]
pub struct IIdentifyWithEvent {
    pub asserted_at: i32,
    pub with_id: i32,
}
impl IIdentifyWithEvent {
    pub fn create_local(conn: &SqliteConnection, with_id: i32) {
        let time = Time::create_local::<Self>(conn);
        insert_into(i_identify_with_event::table)
            .values(&(i_identify_with_event::asserted_at.eq(time), i_identify_with_event::with_id.eq(with_id)))
            .execute(conn)
            .unwrap();
    }
}
impl Event for IIdentifyWithEvent {
    type Arguments = Uuid;
    const EVENT_TYPE: EventType = EventType::IIdentifyWithEvent;

    fn get_arguments(conn: &SqliteConnection, time: i32) -> Self::Arguments {
        let with_id: String = i_identify_with_event::table
            .inner_join(peer::table)
            .select(peer::uuid)
            .filter(i_identify_with_event::asserted_at.eq(time))
            .first(conn)
            .unwrap();
        Uuid::parse_str(&with_id).unwrap()
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
                SELECT time1.peer_id AS left_id, time2.peer_id AS right_id
                FROM i_identify_with_event AS id1 JOIN time AS time1 ON id1.asserted_at = time1.id
                JOIN i_identify_with_event AS id2 ON time1.peer_id = id2.with_id
                JOIN time AS time2 ON id2.asserted_at = time2.id AND time2.peer_id = id1.with_id
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
#[table_name="my_name_is_event"]
#[primary_key(asserted_at)]
#[belongs_to(Time, foreign_key="asserted_at")]
pub struct MyNameIsEvent {
    pub asserted_at: i32,
    pub name: String,
}
impl Event for MyNameIsEvent {
    type Arguments = String;
    const EVENT_TYPE: EventType = EventType::MyNameIsEvent;

    fn get_arguments(conn: &SqliteConnection, time: i32) -> Self::Arguments {
        my_name_is_event::table
            .select(my_name_is_event::name)
            .filter(my_name_is_event::asserted_at.eq(time))
            .first(conn)
            .unwrap()
    }
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name="peer_name"]
#[primary_key(peer_id, asserted_at)]
#[belongs_to(Peer)]
#[belongs_to(Time, foreign_key="asserted_at")]
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
            SELECT sp.right_id AS peer_id, time.id AS asserted_at, lag(time.id) OVER by_peer AS retracted_at, myname.name
            FROM my_name_is_event AS myname
            JOIN time ON myname.asserted_at = time.id
            JOIN same_person AS sp ON sp.left_id = time.peer_id
            LEFT JOIN peer_name old ON old.peer_id = sp.right_id AND old.asserted_at = time.id
            WHERE old.peer_id IS NULL
            WINDOW by_peer AS (
                PARTITION BY sp.right_id ORDER BY time.wall DESC ROWS BETWEEN 1 PRECEDING AND CURRENT ROW
            )
        ").execute(conn).unwrap()
    }
}
