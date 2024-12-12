CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    pubkey BLOB NOT NULL,
    created DATETIME NOT NULL DEFAULT current_timestamp
);

CREATE TABLE host (
    id INTEGER PRIMARY KEY,
    https INT NOT NULL,
    name VARCHAR NOT NULL UNIQUE,
    created DATETIME NOT NULL DEFAULT current_timestamp
);
CREATE UNIQUE INDEX uk_host ON host(https, name);

CREATE TABLE path (
    id INTEGER PRIMARY KEY,
    path VARCHAR NOT NULL UNIQUE,
    created DATETIME NOT NULL DEFAULT current_timestamp
);

CREATE TABLE query (
    id INTEGER PRIMARY KEY,
    query VARCHAR NOT NULL UNIQUE,
    created DATETIME NOT NULL DEFAULT current_timestamp
);

CREATE TABLE link (
    id INTEGER PRIMARY KEY,
    host_id INTEGER REFERENCES host(id),
    path_id INTEGER REFERENCES path(id),
    query_id INTEGER REFERENCES query(id),
    created DATETIME NOT NULL DEFAULT current_timestamp,
    last_check DATETIME NOT NULL DEFAULT current_timestamp
);
CREATE UNIQUE INDEX uk_link ON link(host_id, path_id, query_id);

CREATE TABLE keyword (
    id INTEGER PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    created DATETIME NOT NULL DEFAULT current_timestamp
);

CREATE TABLE link_keyword (
    link_id INTEGER REFERENCES link(id),
    keyword_id INTEGER REFERENCES keyword(id),
    count INTEGER NOT NULL
);

CREATE TABLE link_to (
    link_id INTEGER REFERENCES link(id),
    to_id INTEGER REFERENCES link(id)
);
