ALTER TABLE kvs DROP CONSTRAINT kvs_pkey;
ALTER TABLE kvs ADD CONSTRAINT kvs_pkey PRIMARY KEY (workspace, namespace, key);
