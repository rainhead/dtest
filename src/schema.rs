table! {
    entities (id) {
        id -> Integer,
    }
}

table! {
    events (id) {
        id -> Integer,
        ts -> Timestamp,
        peer -> Integer,
        seq_no -> Integer,
    }
}

table! {
    message (id) {
        id -> Integer,
    }
}

table! {
    message_body (id, ts) {
        id -> Integer,
        ts -> Integer,
        body -> Text,
    }
}

table! {
    peer_name (peer, ts) {
        peer -> Integer,
        ts -> Integer,
        retracted_at -> Nullable<Integer>,
        name -> Text,
    }
}

table! {
    peers (id) {
        id -> Integer,
    }
}

table! {
    send_message_event (ts) {
        ts -> Integer,
        body -> Text,
    }
}

table! {
    send_message_events (event_id) {
        event_id -> Integer,
    }
}

joinable!(events -> peers (peer));
joinable!(message -> entities (id));
joinable!(message_body -> entities (id));
joinable!(message_body -> events (ts));
joinable!(peer_name -> peers (peer));
joinable!(send_message_event -> events (ts));
joinable!(send_message_events -> events (event_id));

allow_tables_to_appear_in_same_query!(
    entities,
    events,
    message,
    message_body,
    peer_name,
    peers,
    send_message_event,
    send_message_events,
);
