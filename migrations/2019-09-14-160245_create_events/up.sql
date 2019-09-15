CREATE TABLE entities (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL
);

CREATE TABLE peers (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL
);

CREATE TABLE events (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL,
    ts TIMESTAMP NOT NULL,
    peer INTEGER NOT NULL REFERENCES peers (id),
    seq_no INTEGER NOT NULL
);
CREATE UNIQUE INDEX events_by_peer ON events (peer, seq_no DESC);

CREATE TABLE send_message_event (
    asserted_at INTEGER PRIMARY KEY NOT NULL REFERENCES events (id),
    body TEXT NOT NULL
);

CREATE TABLE message (
    entity_id INTEGER PRIMARY KEY NOT NULL REFERENCES entities (id)
);

CREATE TABLE message_body (
    entity_id INTEGER NOT NULL REFERENCES entities (id),
    asserted_at INTEGER NOT NULL REFERENCES events (id),
    body TEXT NOT NULL,
    PRIMARY KEY (entity_id, asserted_at)
);

CREATE TABLE peer_name (
    peer_id INTEGER NOT NULL REFERENCES peers (id),
    asserted_at INTEGER NOT NULL REFERENCES events (id),
    retracted_at INTEGER REFERENCES events (id),
    `name` TEXT NOT NULL,
    PRIMARY KEY (peer_id, asserted_at)
);
CREATE INDEX valid_peer_name ON peer_name (peer_id) WHERE retracted_at IS NULL;
