DROP TABLE IF EXISTS authors;

CREATE TABLE authors (
    id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL
);
