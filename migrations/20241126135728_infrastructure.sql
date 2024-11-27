CREATE TABLE metadata(
    idx INTEGER NOT NULL,
    file TEXT NOT NULL,
    mtime DATE NOT NULL
);

CREATE INDEX metadata_index
ON metadata (idx);

CREATE TABLE profiles(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    engine VARCHAR(20) UNIQUE NOT NULL,
    source_path TEXT NOT NULL,
    source_file_extension VARCHAR(10) NOT NULL,
    output_path TEXT NOT NULL,
    output_file_extension VARCHAR(10) NOT NULL,
    options TEXT,
    ignore_patterns TEXT,
    metadata_index INTEGER,
    FOREIGN KEY (metadata_index)
    REFERENCES metadata(idx)
);
