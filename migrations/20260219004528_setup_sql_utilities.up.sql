-- Add up migration script here

-- 1. Create the function that acts as the trigger
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 2. Create the helper function to attach the trigger to a table easily
CREATE OR REPLACE FUNCTION trigger_updated_at(tablename regclass)
RETURNS void AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD IS DISTINCT FROM NEW)
    EXECUTE FUNCTION set_updated_at();', tablename);
END;
$$ LANGUAGE plpgsql;

-- 3. Create the case-insensitive collation
-- "IF NOT EXISTS" is safer for manual re-runs
CREATE COLLATION IF NOT EXISTS case_insensitive (
    provider = icu,
    locale = 'und-u-ks-level2',
    deterministic = false
);
