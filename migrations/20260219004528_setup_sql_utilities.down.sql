-- Add down migration script here

-- 1. Drop the collation
-- We use IF EXISTS to prevent the migration from failing if the object is already gone.
DROP COLLATION IF EXISTS case_insensitive;

-- 2. Drop the helper function
-- specifying the argument type (regclass) ensures we drop the correct overload.
DROP FUNCTION IF EXISTS trigger_updated_at(regclass);

-- 3. Drop the trigger logic function
DROP FUNCTION IF EXISTS set_updated_at();