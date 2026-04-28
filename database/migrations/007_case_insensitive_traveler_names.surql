-- ------------------------------
-- Case-insensitive uniqueness for traveler names within a chat
-- ------------------------------
-- Previously the unique index was `(chat, name)`, allowing names that only
-- differ in case (e.g. "Alice" and "alice") to coexist in the same chat.
-- This migration adds a `name_lower` field (populated by the application on
-- write), backfills it for existing rows, replaces the unique index, and
-- updates `fn::delete_traveler` to look up travelers case-insensitively.

-- 1. New mandatory field. Application code sets it on create/update.
DEFINE FIELD OVERWRITE name_lower ON traveler TYPE string PERMISSIONS FOR select, create, update WHERE FULL;

-- 2. Backfill existing rows.
UPDATE traveler SET name_lower = string::lowercase(name);

-- 3. Replace the unique index with a case-insensitive one.
REMOVE INDEX IF EXISTS traveler_chat_name_index ON traveler;
DEFINE INDEX traveler_chat_name_lower_index ON traveler FIELDS chat, name_lower UNIQUE;

-- 4. Update fn::delete_traveler to match by lowercase name.
DEFINE FUNCTION OVERWRITE fn::delete_traveler($chat: record<chat>, $name: string) {
    LET $traveler = (SELECT * FROM traveler WHERE chat = $chat AND name_lower = string::lowercase($name))[0];
    DELETE ($traveler->paid_for->expense).id;
    DELETE $traveler;
} COMMENT 'Delete a traveler and all expenses paid by them' PERMISSIONS FULL;

