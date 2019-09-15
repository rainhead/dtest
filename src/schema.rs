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
    message (entity_id) {
        entity_id -> Integer,
    }
}

table! {
    message_body (entity_id, asserted_at) {
        entity_id -> Integer,
        asserted_at -> Integer,
        body -> Text,
    }
}

table! {
    peer_name (peer_id, asserted_at) {
        peer_id -> Integer,
        asserted_at -> Integer,
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
    send_message_event (asserted_at) {
        asserted_at -> Integer,
        body -> Text,
    }
}

table! {
    send_message_events (event_id) {
        event_id -> Integer,
    }
}

joinable!(events -> peers (peer));
joinable!(message -> entities (entity_id));
joinable!(message_body -> entities (entity_id));
joinable!(message_body -> events (asserted_at));
joinable!(peer_name -> peers (peer_id));
joinable!(send_message_event -> events (asserted_at));
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
