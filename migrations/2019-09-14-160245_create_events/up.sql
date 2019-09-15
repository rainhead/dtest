CREATE TABLE entity (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL
);

CREATE TABLE peer (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL
);

CREATE TABLE event (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL,
    ts TIMESTAMP NOT NULL,
    peer_id INTEGER NOT NULL REFERENCES peer (id),
    seq_no INTEGER NOT NULL
);
CREATE UNIQUE INDEX event_by_peer ON event (peer_id, seq_no DESC);

CREATE TABLE send_message_event (
    asserted_at INTEGER PRIMARY KEY NOT NULL REFERENCES event (id),
    body TEXT NOT NULL
);

CREATE TABLE message (
    entity_id INTEGER PRIMARY KEY NOT NULL REFERENCES entity (id)
);

CREATE TABLE message_body (
    entity_id INTEGER NOT NULL REFERENCES entity (id),
    asserted_at INTEGER NOT NULL REFERENCES event (id),
    body TEXT NOT NULL,
    PRIMARY KEY (entity_id, asserted_at)
);

CREATE TABLE peer_name (
    peer_id INTEGER NOT NULL REFERENCES peer (id),
    asserted_at INTEGER NOT NULL REFERENCES event (id),
    retracted_at INTEGER REFERENCES event (id),
    `name` TEXT NOT NULL,
    PRIMARY KEY (peer_id, asserted_at)
);
CREATE INDEX valid_peer_name ON peer_name (peer_id) WHERE retracted_at IS NULL;
