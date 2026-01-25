CREATE TABLE profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    engine VARCHAR(20) NOT NULL,
    source_root TEXT NOT NULL,
    source_file_extension VARCHAR(10) NOT NULL,
    output_root TEXT NOT NULL,
    output_file_extension VARCHAR(10) NOT NULL,
    options TEXT,
    ignore_patterns TEXT
);

CREATE TABLE metadata (
    profile_id INTEGER NOT NULL,
    source_file TEXT NOT NULL,
    mtime INTEGER NOT NULL,
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, source_file)
);

CREATE INDEX idx_engine ON profiles (engine);
CREATE INDEX idx_metadata_profile_id ON metadata (profile_id);
