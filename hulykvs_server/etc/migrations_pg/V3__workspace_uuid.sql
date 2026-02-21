DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'kvs'
          AND column_name = 'workspace'
    ) THEN
        IF EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON c.conrelid = t.oid
            JOIN pg_namespace n ON t.relnamespace = n.oid
            WHERE c.contype = 'p'
              AND c.conname = 'kvs_pkey'
              AND t.relname = 'kvs'
              AND n.nspname = current_schema()
              AND pg_get_constraintdef(c.oid) LIKE 'PRIMARY KEY (workspace, namespace, key)%'
        ) THEN
            RETURN;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON c.conrelid = t.oid
            JOIN pg_namespace n ON t.relnamespace = n.oid
            WHERE c.contype = 'p'
              AND c.conname = 'kvs_pkey'
              AND t.relname = 'kvs'
              AND n.nspname = current_schema()
        ) THEN
            ALTER TABLE kvs DROP CONSTRAINT kvs_pkey;
        END IF;

        ALTER TABLE kvs ADD CONSTRAINT kvs_pkey PRIMARY KEY (workspace, namespace, key);
    END IF;
END $$;
