# Database Schema Management

This directory contains the schema and migration scripts for the Travel-RS Bot database.

## Building the Schema

To create or completely rebuild the database schema from scratch, use the [`build_travelers_db.surql`](build_travelers_db.surql) script.  
This script **overwrites all existing table, field, and function definitions**, ensuring a clean and consistent schema state.

**Usage:**  
Run the script in your SurrealDB instance to initialize or reset the schema.

## Applying Migrations

For incremental updates or changes to the schema, use the migration scripts in the [`migrations/`](migrations/) folder.  
Each migration script contains changes that should be applied in order (by filename/number) to update the schema without overwriting existing data or definitions.

**Usage:**  
Execute each migration script in the order they appear (e.g., `001_init.surql`, `002_add_timestamps.surql`, etc.).

## Summary

- Use [`build_travelers_db.surql`](build_travelers_db.surql) to build or reset the entire schema (destructive, overwrites all definitions).
- Use scripts in [`migrations/`](migrations/) for incremental, non-destructive schema updates (apply in order).

**Note:** Always back up your data before running schema changes, especially when using the full build script.