CREATE TABLE IF NOT EXISTS cmd (
    alias TEXT PRIMARY KEY,
    cmd TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS python (
    alias TEXT PRIMARY KEY,
    `path` TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS javascript (
    alias TEXT PRIMARY KEY,
    `path` TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS powershell (
    alias TEXT PRIMARY KEY,
    `path` TEXT NOT NULL
);
