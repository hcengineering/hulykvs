-- ALTER TABLE kvs DROP CONSTRAINT kvs_namespace_key_key;
-- DROP INDEX kvs_namespace_key_key;
DROP INDEX kvs_namespace_key_key CASCADE;

