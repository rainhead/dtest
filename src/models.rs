use crate::schema::events;

#[derive(Queryable)]
pub struct Entity {
    pub id: i32,
}

#[derive(Queryable)]
pub struct Peer {
    pub id: i32,
}

#[derive(Queryable)]
pub struct Event {
    pub id: i32,
    pub ts: chrono::NaiveDateTime,
}

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

#[derive(Queryable)]
pub struct SendMessageEvent {
    pub ts: i32,
    pub body: String,
}

#[derive(Queryable)]
pub struct Message {
    pub id: i32,
}

#[derive(Queryable)]
pub struct MessageBody {
    pub id: i32,
    pub ts: i32,
    pub body: String,
}

#[derive(Queryable)]
pub struct PeerName {
    pub peer: i32,
    pub ts: i32,
    pub retracted_at: i32,
    pub name: String,
}
