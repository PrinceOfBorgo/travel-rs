-- ------------------------------
-- Chat-equality assertions on relation tables
-- ------------------------------
-- Enforce that both endpoints of every relation belong to the same chat.
-- This is a defense-in-depth constraint: the application already guarantees
-- this, but a direct DB write (admin tool, migration script, etc.) could
-- accidentally create cross-chat relationships, leading to corrupted
-- balances and debt calculations.
--
-- Relations affected:
--   * owes            (traveler -> traveler) : in.chat == out.chat
--   * transferred_to  (traveler -> traveler) : in.chat == out.chat
--   * paid_for        (traveler -> expense)  : in.chat == out.chat
--   * split           (traveler -> expense)  : in.chat == out.chat

-- owes
DEFINE FIELD OVERWRITE in ON owes TYPE record<traveler>
    ASSERT $value.chat == $this.out.chat
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON owes TYPE record<traveler>
    ASSERT $value.chat == $this.in.chat
    PERMISSIONS FULL;

-- transferred_to (preserves existing sender != receiver assertion)
DEFINE FIELD OVERWRITE in ON transferred_to TYPE record<traveler>
    ASSERT $value != $this.out
        AND $value.chat == $this.out.chat
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON transferred_to TYPE record<traveler>
    ASSERT $value != $this.in
        AND $value.chat == $this.in.chat
    PERMISSIONS FULL;

-- paid_for
DEFINE FIELD OVERWRITE in ON paid_for TYPE record<traveler>
    ASSERT $value.chat == $this.out.chat
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON paid_for TYPE record<expense>
    ASSERT $value.chat == $this.in.chat
    PERMISSIONS FULL;

-- split
DEFINE FIELD OVERWRITE in ON split TYPE record<traveler>
    ASSERT $value.chat == $this.out.chat
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON split TYPE record<expense>
    ASSERT $value.chat == $this.in.chat
    PERMISSIONS FULL;
