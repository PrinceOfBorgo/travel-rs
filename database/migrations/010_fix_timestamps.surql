-- `timestamp_utc` was defined with `VALUE time::now()`, which SurrealDB
-- re-evaluates on EVERY write: any UPDATE of an expense or transfer (a data
-- backfill, a future edit command, ...) silently reset its date to "now".
-- `DEFAULT ... READONLY` sets it once on creation and keeps it afterwards.
-- Existing values are left untouched.
DEFINE FIELD OVERWRITE timestamp_utc ON expense TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FOR select FULL, FOR create, update NONE;
DEFINE FIELD OVERWRITE timestamp_utc ON transferred_to TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FOR select FULL, FOR create, update NONE;
