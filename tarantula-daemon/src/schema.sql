
CREATE TABLE host (
    id INTEGER PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    created DATETIME NOT NULL DEFAULT current_timestamp
);

CREATE TABLE path (
    id INTEGER PRIMARY KEY,
    path VARCHAR NOT NULL UNIQUE,
    created DATETIME NOT NULL DEFAULT current_timestamp
);

CREATE TABLE host_path (
    id INTEGER PRIMARY KEY,
    host_id INTEGER REFERENCES host(id),
    path_id INTEGER REFERENCES path(id),
    created DATETIME NOT NULL DEFAULT current_timestamp,
    last_check DATETIME NOT NULL DEFAULT current_timestamp
);
CREATE UNIQUE INDEX uk_hp ON host_path(host_id, path_id);

CREATE TABLE keyword (
    id INTEGER PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE host_path_keyword (
    host_path_id INTEGER REFERENCES host_path(id),
    keyword_id INTEGER REFERENCES keyword(id),
    count INTEGER NOT NULL
);

CREATE TABLE host_path_link (
    host_path_id INTEGER REFERENCES host_path(id),
    link_id INTEGER REFERENCES host_path(id)
);
