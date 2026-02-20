DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'kvs'
          AND column_name = 'workspace'
    ) THEN
        ALTER TABLE kvs ALTER COLUMN workspace DROP DEFAULT;
    END IF;
END $$;
