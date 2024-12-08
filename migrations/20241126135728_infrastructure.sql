CREATE TABLE metadata(
    id INTEGER NOT NULL,
    file TEXT NOT NULL,
    mtime DATE NOT NULL
);

CREATE INDEX idx_metadata_id
ON metadata (id);

CREATE TABLE profiles(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    engine VARCHAR(20) NOT NULL,
    source_root TEXT NOT NULL,
    source_file_extension VARCHAR(10) NOT NULL,
    output_root TEXT NOT NULL,
    output_file_extension VARCHAR(10) NOT NULL,
    options TEXT,
    ignore_patterns TEXT
);

CREATE INDEX idx_engine
ON profiles (engine);
