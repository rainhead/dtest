use crate::schema::*;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name="entities"]
pub struct Entity {
    pub id: i32,
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
pub struct Peer {
    pub id: i32,
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
pub struct Event {
    pub id: i32,
    pub ts: chrono::NaiveDateTime,
}

// workaround per https://github.com/diesel-rs/diesel/issues/89
pub struct Retraction(pub Event);

#[derive(Insertable)]
#[table_name="events"]
pub struct LocalEvent {
    pub ts: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name="events"]
pub struct IncomingEvent {
    pub ts: chrono::NaiveDateTime,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="send_message_event"]
#[belongs_to(Event, foreign_key="asserted_at")]
#[primary_key(asserted_at)]
pub struct SendMessageEvent {
    pub asserted_at: i32,
    pub body: String,
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
