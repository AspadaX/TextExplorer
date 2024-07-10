-- Your SQL goes here
CREATE TABLE documents (
    id VARCHAR(36) PRIMARY KEY,
    title VARCHAR(255),
    full_text TEXT,
    created_at DATETIME
);