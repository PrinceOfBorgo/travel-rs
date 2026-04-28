-- ------------------------------
-- Defense-in-depth validation constraints
-- ------------------------------
-- Mirror the application-level input validation rules at the schema level so
-- that any direct DB write (admin tool, future ingestion path, etc.) cannot
-- bypass them.
--
-- Limits / rules enforced:
--   * traveler.name           : non-empty, must not start with '/',
--                               must not contain any of ';' ':' ','
--   * expense.amount          : > 0
--   * split.amount            : > 0
--   * transferred_to.amount   : > 0
--   * transferred_to in/out   : sender != receiver
--   * chat.currency           : non-empty, uppercase letters only
--
-- NOTE: Reserved-keyword and control-character checks remain application-only
-- (they would require maintaining a duplicated keyword list in SQL and a
-- portable control-character regex, both of which add maintenance burden for
-- little extra defense). Length caps and description-newline rules are
-- intentionally not enforced.

-- traveler.name: enforce slash prefix and invalid chars
DEFINE FIELD OVERWRITE name ON traveler TYPE string
    ASSERT string::len($value) > 0
        AND !string::starts_with($value, "/")
        AND !string::contains($value, ";")
        AND !string::contains($value, ":")
        AND !string::contains($value, ",")
        AND !string::contains($value, '"')
    PERMISSIONS FOR select, create, update WHERE FULL;

-- Positive monetary amounts (expense, split share, transfer)
DEFINE FIELD OVERWRITE amount ON expense TYPE decimal
    ASSERT $value > 0dec
    PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE FIELD OVERWRITE amount ON split TYPE decimal
    ASSERT $value > 0dec
    PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE FIELD OVERWRITE amount ON transferred_to TYPE decimal
    ASSERT $value > 0dec
    PERMISSIONS FOR select, create, update WHERE FULL;

-- transferred_to: sender (in) and receiver (out) must differ
DEFINE FIELD OVERWRITE in ON transferred_to TYPE record<traveler>
    ASSERT $value != $this.out
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON transferred_to TYPE record<traveler>
    ASSERT $value != $this.in
    PERMISSIONS FULL;

-- chat.currency: non-empty, uppercase letters only (e.g. USD, EUR, BTC)
DEFINE FIELD OVERWRITE currency ON chat TYPE string
    ASSERT string::len($value) > 0
        AND string::matches($value, "^[A-Z]+$")
    PERMISSIONS FOR select, create, update WHERE FULL;
