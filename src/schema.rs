table! {
    entity (id) {
        id -> Integer,
        uuid -> Text,
        introduced_at -> Integer,
    }
}

table! {
    i_identify_with_event (asserted_at) {
        asserted_at -> Integer,
        with_id -> Integer,
    }
}

table! {
    identify_with_event (asserted_at) {
        asserted_at -> Integer,
        with_id -> Integer,
    }
}

table! {
    message (entity_id) {
        entity_id -> Integer,
    }
}

table! {
    message_author (entity_id) {
        entity_id -> Integer,
        asserted_at -> Integer,
        peer_id -> Integer,
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
    mutually_identify (left_id, right_id) {
        left_id -> Integer,
        right_id -> Integer,
    }
}

table! {
    my_name_is_event (asserted_at) {
        asserted_at -> Integer,
        name -> Text,
    }
}

table! {
    peer (id) {
        id -> Integer,
        uuid -> Text,
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
    peer_name_event (asserted_at) {
        asserted_at -> Integer,
        name -> Text,
    }
}

table! {
    same_person (left_id, right_id) {
        left_id -> Integer,
        right_id -> Integer,
    }
}

table! {
    send_message_event (asserted_at) {
        asserted_at -> Integer,
        message_id -> Integer,
        body -> Text,
    }
}

table! {
    send_message_events (event_id) {
        event_id -> Integer,
    }
}

table! {
    time (id) {
        id -> Integer,
        wall -> Timestamp,
        peer_id -> Integer,
        seq_no -> Integer,
        event_type -> Text,
    }
}

joinable!(entity -> time (introduced_at));
joinable!(i_identify_with_event -> peer (with_id));
joinable!(i_identify_with_event -> time (asserted_at));
joinable!(identify_with_event -> peer (with_id));
joinable!(identify_with_event -> time (asserted_at));
joinable!(message -> entity (entity_id));
joinable!(message_author -> entity (entity_id));
joinable!(message_author -> peer (peer_id));
joinable!(message_author -> time (asserted_at));
joinable!(message_body -> entity (entity_id));
joinable!(message_body -> time (asserted_at));
joinable!(my_name_is_event -> time (asserted_at));
joinable!(peer_name -> peer (peer_id));
joinable!(peer_name_event -> time (asserted_at));
joinable!(send_message_event -> entity (message_id));
joinable!(send_message_event -> time (asserted_at));
joinable!(time -> peer (peer_id));

allow_tables_to_appear_in_same_query!(
    entity,
    i_identify_with_event,
    identify_with_event,
    message,
    message_author,
    message_body,
    mutually_identify,
    my_name_is_event,
    peer,
    peer_name,
    peer_name_event,
    same_person,
    send_message_event,
    send_message_events,
    time,
);
