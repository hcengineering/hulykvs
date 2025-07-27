-- 1. Create new
CREATE TABLE kvs_new (
    workspace TEXT NOT NULL,
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    md5 BYTES NOT NULL,
    value BYTES NOT NULL,
    PRIMARY KEY (workspace, namespace, key)
);

-- 2. Copy
INSERT INTO kvs_new (workspace, namespace, key, md5, value)
SELECT 'defaultspace', namespace, key, md5, value FROM kvs;

-- 3. Del
DROP TABLE kvs;

-- 4. Rename
ALTER TABLE kvs_new RENAME TO kvs;
