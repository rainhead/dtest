table! {
    entity (id) {
        id -> Integer,
    }
}

table! {
    event (id) {
        id -> Integer,
        ts -> Timestamp,
        peer_id -> Integer,
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
    peer (id) {
        id -> Integer,
        is_local -> Bool,
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

joinable!(event -> peer (peer_id));
joinable!(message -> entity (entity_id));
joinable!(message_body -> entity (entity_id));
joinable!(message_body -> event (asserted_at));
joinable!(peer_name -> peer (peer_id));
joinable!(send_message_event -> event (asserted_at));

allow_tables_to_appear_in_same_query!(
    entity,
    event,
    message,
    message_body,
    peer,
    peer_name,
    send_message_event,
    send_message_events,
);
