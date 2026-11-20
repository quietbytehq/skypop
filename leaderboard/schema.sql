CREATE TABLE IF NOT EXISTS scores (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 name TEXT NOT NULL,
 score INTEGER NOT NULL,
 mode TEXT NOT NULL,
 version TEXT NOT NULL,
 created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS scores_rank ON scores(name, score DESC);
-- Separate from scores so moderation cannot reset a caller's allowance.
CREATE TABLE IF NOT EXISTS submissions (
 id TEXT PRIMARY KEY,
 ip_hash TEXT NOT NULL,
 created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS submissions_rate ON submissions(ip_hash, created_at);
