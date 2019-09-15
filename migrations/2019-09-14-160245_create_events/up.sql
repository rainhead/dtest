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
    ts INTEGER PRIMARY KEY NOT NULL REFERENCES events (id),
    body TEXT NOT NULL
);

CREATE TABLE message (
    id INTEGER PRIMARY KEY NOT NULL REFERENCES entities (id)
);

CREATE TABLE message_body (
    id INTEGER NOT NULL REFERENCES entities (id),
    ts INTEGER NOT NULL REFERENCES events (id),
    body TEXT NOT NULL,
    PRIMARY KEY (id, ts)
);

CREATE TABLE peer_name (
    peer INTEGER NOT NULL REFERENCES peers (id),
    ts INTEGER NOT NULL REFERENCES events (id),
    retracted_at INTEGER REFERENCES events (id),
    `name` TEXT NOT NULL,
    PRIMARY KEY (peer, ts)
);
CREATE INDEX valid_peer_name ON peer_name (peer) WHERE retracted_at IS NULL;
