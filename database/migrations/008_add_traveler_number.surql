-- Add a progressive numeric ID to travelers (auto-incremented per chat,
-- same pattern as expense.number and transferred_to.number).
DEFINE FIELD OVERWRITE number ON traveler TYPE int PERMISSIONS FOR select, create, update WHERE FULL;

-- Assign numbers to existing travelers.
-- This is a one-time backfill for travelers created before this migration.
LET $chats = (SELECT VALUE id FROM chat);
FOR $chat IN $chats {
    LET $travelers = (SELECT VALUE id FROM traveler WHERE chat = $chat);
    FOR $i IN 0..array::len($travelers) {
        UPDATE $travelers[$i] SET number = $i + 1;
    };
};

DEFINE INDEX OVERWRITE traveler_chat_number_index ON traveler FIELDS chat, number UNIQUE;
