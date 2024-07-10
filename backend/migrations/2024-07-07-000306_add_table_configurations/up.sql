-- Your SQL goes here
CREATE TABLE configurations (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    service TEXT NOT NULL,
    version TEXT NOT NULL,
    data TEXT NOT NULL
);