CREATE TABLE entity (
    id INTEGER PRIMARY KEY NOT NULL,
    uuid TEXT UNIQUE NOT NULL,
    introduced_at INTEGER NOT NULL REFERENCES time (id)
);

CREATE TABLE peer (
    id INTEGER PRIMARY KEY NOT NULL,
    uuid TEXT UNIQUE NOT NULL,
    is_local BOOLEAN NOT NULL DEFAULT 0
);
-- this appears not to enforce a unique record or is_local
CREATE UNIQUE INDEX local_peer ON peer (id) WHERE is_local;

CREATE TABLE time (
    id INTEGER PRIMARY KEY NOT NULL,
    wall TIMESTAMP NOT NULL,
    peer_id INTEGER NOT NULL REFERENCES peer (id),
    seq_no INTEGER NOT NULL,
    event_type TEXT NOT NULL
);
CREATE UNIQUE INDEX time_by_peer ON time (peer_id, seq_no DESC);

CREATE TABLE send_message_event (
    asserted_at INTEGER PRIMARY KEY NOT NULL REFERENCES time (id),
    message_id INTEGER UNIQUE NOT NULL REFERENCES entity (id),
    body TEXT NOT NULL
);

CREATE TABLE message (
    entity_id INTEGER PRIMARY KEY NOT NULL REFERENCES entity (id)
);

CREATE TABLE message_body (
    entity_id INTEGER NOT NULL REFERENCES entity (id),
    asserted_at INTEGER NOT NULL REFERENCES time (id),
    body TEXT NOT NULL,
    PRIMARY KEY (entity_id, asserted_at)
);

CREATE TABLE message_author (
    entity_id INTEGER PRIMARY KEY NOT NULL REFERENCES entity (id),
    asserted_at INTEGER NOT NULL REFERENCES time (id),
    peer_id INTEGER NOT NULL REFERENCES peer (id)
);

CREATE TABLE message_view (
    entity_id INTEGER PRIMARY KEY NOT NULL REFERENCES entity (id),
    author_name TEXT,
    body TEXT NOT NULL,
    sent_at TIMESTAMP NOT NULL
);

CREATE TABLE i_identify_with_event (
    asserted_at INTEGER PRIMARY KEY NOT NULL REFERENCES time (id),
    with_id INTEGER NOT NULL REFERENCES peer (id)
);

CREATE TABLE mutually_identify (
    left_id INTEGER NOT NULL REFERENCES peer (id),
    right_id INTEGER NOT NULL REFERENCES peer (id),
    PRIMARY KEY (left_id, right_id)
);

CREATE TABLE same_person (
    left_id INTEGER NOT NULL REFERENCES peer (id),
    right_id INTEGER NOT NULL REFERENCES peer (id),
    PRIMARY KEY (left_id, right_id)
);

CREATE TABLE my_name_is_event (
    asserted_at INTEGER PRIMARY KEY NOT NULL REFERENCES time (id),
    name TEXT NOT NULL
);

CREATE TABLE peer_name (
    peer_id INTEGER NOT NULL REFERENCES peer (id),
    asserted_at INTEGER NOT NULL REFERENCES time (id),
    retracted_at INTEGER REFERENCES time (id),
    `name` TEXT NOT NULL,
    PRIMARY KEY (peer_id, asserted_at)
);
CREATE INDEX valid_peer_name ON peer_name (peer_id) WHERE retracted_at IS NULL;
