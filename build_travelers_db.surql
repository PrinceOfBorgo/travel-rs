-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- FUNCTIONS
-- ------------------------------

DEFINE FUNCTION fn::calc_debts($chat: record<chat>) -> any {
    // Calculate individual shares within the specified chat.
    // 'split' table likely represents individual expenses split between users.
    LET $shares = (
        SELECT 
            in AS debtor, // The user who owes money.
            (out<-paid_for<-traveler)[0] AS creditor, // The user who paid for the expense.  Assumes a relationship path.
            amount // The amount owed.
        FROM split 
        WHERE in.chat.id = $chat // Filter by chat ID.
    );

    // Retrieve transfer records within the specified chat.
    // 'transferred_to' likely represents direct payments between users.
    LET $transfers = (
        SELECT 
            in AS from, // The user who sent the transfer.
            out AS to, // The user who received the transfer.
            amount // The amount transferred.
        FROM transferred_to 
        WHERE in.chat.id = $chat // Filter by chat ID.
    );

    // Aggregate debts based on individual shares.
    // Calculates the total debt between each pair of users based on shared expenses.
    LET $debts = (
        SELECT 
            debtor, 
            creditor, 
            math::sum(amount) AS debt // Sums up the amounts owed for each debtor-creditor pair.
        FROM $shares 
        WHERE debtor != creditor // Excludes debts where debtor and creditor are the same.
        GROUP BY debtor, creditor // Groups the results by debtor and creditor.
    );

    // Aggregate transfers between users.
    // Calculates the total amount transferred between each pair of users.
    LET $tot_transfers = (
        SELECT 
            from, 
            to, 
            math::sum(amount) AS transfer // Sums up the transfers for each sender-receiver pair.
        FROM $transfers 
        WHERE from != to // Excludes transfers where sender and receiver are the same.
        GROUP BY from, to // Groups the results by sender and receiver.
    );

    // Calculate the final debts after considering transfers that offset existing debts.
    // Subtracts direct transfers from the initial debts calculated from shares.
    LET $net_debts_from_shares = (
        SELECT 
            debtor, 
            creditor, 
            debt - math::sum((SELECT VALUE transfer FROM $tot_transfers WHERE from = $parent.debtor AND to = $parent.creditor)) AS debt // Subtracts relevant transfers.
        FROM $debts
    );

    // Identify transfers that represent debts where no corresponding share-based debt exists.
    // Handles cases where users have transferred money without a pre-existing split expense.
    LET $transfer_only_debts = (
        SELECT 
            from AS debtor, 
            to AS creditor, 
            -transfer AS debt // Represents the transfer as a debt (negative amount as it reduces what's owed).
        FROM $tot_transfers 
        WHERE !(SELECT * FROM $debts WHERE debtor = $parent.from AND creditor = $parent.to) // Checks if a corresponding debt from shares exists.
    );

    // Combine the adjusted debts and the transfer-only debts to get the final consolidated debt list.
    LET $union_debts = array::union($net_debts_from_shares, $transfer_only_debts);

    // Select all debts that are positive from the union of debts (ignore zero debts)
    LET $positive_debts = (SELECT * FROM $union_debts WHERE debt > 0);

    // Select all debts that are negative, but invert them to be positive and swap debtor and creditor (ignore zero debts)
    LET $negative_to_positive_debts = (SELECT creditor AS debtor, debtor AS creditor, -debt AS debt FROM $union_debts WHERE debt < 0);

    // Sum the debts for each debtor-creditor pair from the union of positive debts and inverted negative debts (ignore zero debts)
    LET $sum_debts = (SELECT debtor, creditor, math::sum(debt) AS debt FROM array::union($positive_debts, $negative_to_positive_debts) GROUP BY debtor, creditor);

    // Return the summed debts
    RETURN $sum_debts;
} COMMENT 'Calculate and simplify debts between users in a chat' PERMISSIONS FULL;
DEFINE FUNCTION fn::clear_debts($chat: record<chat>) { DELETE owes WHERE in.chat = $chat; } PERMISSIONS FULL;
DEFINE FUNCTION fn::get_balances($chat: record<chat>) { RETURN SELECT in.name AS debtor_name, out.name AS creditor_name, amount AS debt, in.chat AS chat FROM owes WHERE in.chat.id = $chat; } COMMENT 'Retrieve the balances among travelers in the specified chat' PERMISSIONS FULL;
DEFINE FUNCTION fn::get_expense_details($chat: record<chat>, $expense_number: int) { RETURN (SELECT amount AS expense_amount, number AS expense_number, description AS expense_description, chat AS chat, (<-paid_for<-traveler)[0].name AS creditor_name, array::group((SELECT in.name AS traveler_name, amount FROM split WHERE out = $parent.id ORDER BY amount DESC, traveler_name ASC)) AS shares FROM expense WHERE chat = $chat AND number = $expense_number); } COMMENT 'Retrieve the details of the expense with the given identifying number in the specified chat' PERMISSIONS FULL;
DEFINE FUNCTION fn::get_transfers($chat: record<chat>) { RETURN SELECT number, amount, in.name AS sender_name, out.name AS receiver_name, in.chat AS chat FROM transferred_to WHERE in.chat.id = $chat; } COMMENT 'Retrieve the transfers recorded in the specified chat' PERMISSIONS FULL;


-- ------------------------------
-- TABLE: chat
-- ------------------------------

DEFINE TABLE chat TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD currency ON chat TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD lang ON chat TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD last_interaction_utc ON chat TYPE datetime PERMISSIONS FOR select, create, update WHERE FULL;


-- ------------------------------
-- TABLE: expense
-- ------------------------------

DEFINE TABLE expense TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD amount ON expense TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD chat ON expense TYPE record<chat> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD description ON expense TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number ON expense TYPE int PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE INDEX expense_chat_number_index ON expense FIELDS chat, number UNIQUE;


-- ------------------------------
-- TABLE: owes
-- ------------------------------

DEFINE TABLE owes TYPE RELATION IN traveler OUT traveler SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD amount ON owes TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD in ON owes TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD out ON owes TYPE record<traveler> PERMISSIONS FULL;

DEFINE INDEX owes_unique_relationships ON owes FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: paid_for
-- ------------------------------

DEFINE TABLE paid_for TYPE RELATION IN traveler OUT expense SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD in ON paid_for TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD out ON paid_for TYPE record<expense> PERMISSIONS FULL;

DEFINE INDEX paid_for_unique_relationships ON paid_for FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: split
-- ------------------------------

DEFINE TABLE split TYPE RELATION IN traveler OUT expense SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD amount ON split TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD in ON split TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD out ON split TYPE record<expense> PERMISSIONS FULL;

DEFINE INDEX split_unique_relationships ON split FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: transferred_to
-- ------------------------------

DEFINE TABLE transferred_to TYPE RELATION IN traveler OUT traveler SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD amount ON transferred_to TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD in ON transferred_to TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD number ON transferred_to TYPE int PERMISSIONS FULL;
DEFINE FIELD out ON transferred_to TYPE record<traveler> PERMISSIONS FULL;

DEFINE INDEX transferred_to_chat_number_index ON transferred_to FIELDS in.chat, number UNIQUE;


-- ------------------------------
-- TABLE: traveler
-- ------------------------------

DEFINE TABLE traveler TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD chat ON traveler TYPE record<chat> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON traveler TYPE string ASSERT string::len($value) > 0 PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE INDEX traveler_chat_name_index ON traveler FIELDS chat, name UNIQUE;
