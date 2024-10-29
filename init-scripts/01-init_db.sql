-- init-scripts/01-init_db.sql

-- Create application user if not exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_user WHERE usename = 'app') THEN
        CREATE USER app WITH PASSWORD 'secret';
        -- Log success
        RAISE NOTICE 'Created user: app';
    END IF;
END
$$;

-- Grant database creation privileges
ALTER USER app CREATEDB;

-- Grant connection privileges
GRANT CONNECT ON DATABASE newsletter TO app;

-- Connect to the database
\c newsletter

-- Grant schema privileges (run after connecting to newsletter database)
GRANT ALL ON SCHEMA public TO app;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO app;
