-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- FUNCTIONS
-- ------------------------------

DEFINE FUNCTION OVERWRITE fn::calc_debts($chat: record<chat>) {
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
DEFINE FUNCTION OVERWRITE fn::clear_debts($chat: record<chat>) { DELETE owes WHERE in.chat.id = $chat; } COMMENT 'Utility function to clear all debts' PERMISSIONS FULL;
DEFINE FUNCTION OVERWRITE fn::delete_traveler($chat: record<chat>, $name: string) {
    LET $traveler = (SELECT * FROM traveler WHERE chat = $chat AND name = $name)[0];
    DELETE ($traveler->paid_for->expense).id;
    DELETE $traveler;
} COMMENT 'Delete a traveler and all expenses paid by them' PERMISSIONS FULL;
DEFINE FUNCTION OVERWRITE fn::get_balances($chat: record<chat>) { RETURN SELECT in.name AS debtor_name, out.name AS creditor_name, amount AS debt, in.chat AS chat FROM owes WHERE in.chat.id = $chat; } COMMENT 'Retrieve the balances among travelers in the specified chat' PERMISSIONS FULL;
DEFINE FUNCTION OVERWRITE fn::get_expense_details($chat: record<chat>, $expense_number: int) { RETURN (SELECT amount AS expense_amount, number AS expense_number, description AS expense_description, chat AS chat, timestamp_utc as timestamp_utc, (<-paid_for<-traveler)[0].name AS creditor_name, array::group((SELECT in.name AS traveler_name, amount FROM split WHERE out = $parent.id ORDER BY amount DESC, traveler_name ASC)) AS shares FROM expense WHERE chat = $chat AND number = $expense_number); } COMMENT 'Retrieve the details of the expense with the given identifying number in the specified chat' PERMISSIONS FULL;
DEFINE FUNCTION OVERWRITE fn::get_transfers($chat: record<chat>) { RETURN SELECT number, amount, in.name AS sender_name, out.name AS receiver_name, in.chat AS chat, timestamp_utc FROM transferred_to WHERE in.chat.id = $chat; } COMMENT 'Retrieve the transfers recorded in the specified chat' PERMISSIONS FULL;
DEFINE FUNCTION OVERWRITE fn::expense_stats($chat: record<chat>) {
    LET $expenses = SELECT * FROM expense WHERE chat = $chat;
    LET $expenses_count = array::len($expenses);
    
    -- Handle case where there are no expenses for the chat
    IF $expenses_count = 0 {
        RETURN {
            expenses_count: 0,
            sum: 0dec,
            mean: 0dec,
            min_expenses: [],
            max_expenses: [],
            average_per_day: None,
            oldest_expense: None,
            newest_expense: None,
        };
    };

    -- sum
    LET $sum = math::sum(SELECT VALUE amount FROM $expenses);
    
    -- mean
    LET $mean = $sum / $expenses_count;

    -- min
    LET $min_amount_val = math::min(SELECT VALUE amount FROM $expenses);
    LET $min_expenses = SELECT * FROM $expenses WHERE amount = $min_amount_val;

    -- max
    LET $max_amount_val = math::max(SELECT VALUE amount FROM $expenses);
    LET $max_expenses = SELECT * FROM $expenses WHERE amount = $max_amount_val;

    LET $oldest_timestamp = time::min(SELECT VALUE timestamp_utc FROM $expenses);
    LET $oldest_expense = (SELECT * FROM $expenses WHERE timestamp_utc = $oldest_timestamp)[0];
    LET $newest_expense = (SELECT * FROM $expenses ORDER BY timestamp_utc DESC)[0];

    -- Calculate the average expense over time, from the oldest expense to now
    LET $current_timestamp = time::now();
    LET $diff_days = time::day($current_timestamp) - time::day($oldest_timestamp) + 1;
    LET $average_per_day = $sum / $diff_days;
    
    RETURN {
        expenses_count: $expenses_count,
        sum: $sum,
        mean: $mean,
        min_expenses: $min_expenses,
        max_expenses: $max_expenses,
        average_per_day: {
            amount: $average_per_day, 
            oldest_timestamp: $oldest_timestamp, 
            now: $current_timestamp,
        },
        oldest_expense: $oldest_expense,
        newest_expense: $newest_expense,
    };
} COMMENT "Calculates various expense statistics (sum, mean, min, max, and time-based average) for a given chat." PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::transfer_stats($chat: record<chat>) {
    LET $transfers = fn::get_transfers($chat);
    LET $transfers_count = array::len($transfers);
    
    -- Handle case where there are no transfers for the chat
    IF $transfers_count = 0 {
        RETURN {
            transfers_count: 0,
            sum: 0dec,
            mean: 0dec,
            min_transfers: [],
            max_transfers: [],
            average_per_day: None,
            oldest_transfer: None,
            newest_transfer: None,
        };
    };

    -- sum
    LET $sum = math::sum(SELECT VALUE amount FROM $transfers);
    
    -- mean
    LET $mean = $sum / $transfers_count;

    -- min
    LET $min_amount_val = math::min(SELECT VALUE amount FROM $transfers);
    LET $min_transfers = SELECT * FROM $transfers WHERE amount = $min_amount_val;

    -- max
    LET $max_amount_val = math::max(SELECT VALUE amount FROM $transfers);
    LET $max_transfers = SELECT * FROM $transfers WHERE amount = $max_amount_val;

    LET $oldest_timestamp = time::min(SELECT VALUE timestamp_utc FROM $transfers);
    LET $oldest_transfer = (SELECT * FROM $transfers WHERE timestamp_utc = $oldest_timestamp)[0];
    LET $newest_transfer = (SELECT * FROM $transfers ORDER BY timestamp_utc DESC)[0];

    -- Calculate the average transfer over time, from the oldest transfer to now
    LET $current_timestamp = time::now();
    LET $diff_days = time::day($current_timestamp) - time::day($oldest_timestamp) + 1;
    LET $average_per_day = $sum / $diff_days;
    
    RETURN {
        transfers_count: $transfers_count,
        sum: $sum,
        mean: $mean,
        min_transfers: $min_transfers,
        max_transfers: $max_transfers,
        average_per_day: {
            amount: $average_per_day, 
            oldest_timestamp: $oldest_timestamp, 
            now: $current_timestamp,
        },
        oldest_transfer: $oldest_transfer,
        newest_transfer: $newest_transfer,
    };
} COMMENT "Calculates various transfer statistics (sum, mean, min, max, and time-based average) for a given chat." PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::traveler_stats($chat: record<chat>) {
    LET $expenses = SELECT amount, (<-paid_for<-traveler)[0] AS creditor FROM expense WHERE chat = $chat;
    LET $travelers_count = array::len(SELECT * FROM traveler WHERE chat = $chat);

    -- Handle case where there are no expenses for the chat
    IF array::len($expenses) = 0 {
        RETURN {
            travelers_count: $travelers_count,
            travelers_paid_most: [],
            travelers_paid_least: [],
            travelers_pays_most_frequently: [],
            travelers_pays_least_frequently: [],
            major_debtors: [],
            major_creditors: [],
        };
    };

    -- Group expenses by traveler to calculate individual statistics
    LET $traveler_payments = SELECT 
        creditor,
        creditor.name AS traveler_name,
        math::sum(amount) AS total_paid,
        count() AS payment_count
    FROM $expenses
    GROUP BY creditor
    ORDER BY traveler_name;

    -- Travelers who paid the most
    LET $max_paid_amount = math::max(SELECT VALUE total_paid FROM $traveler_payments);
    LET $travelers_paid_most = SELECT traveler_name, total_paid AS amount FROM $traveler_payments WHERE total_paid = $max_paid_amount;

    -- Travelers who paid the least
    LET $min_paid_amount = math::min(SELECT VALUE total_paid FROM $traveler_payments);
    LET $travelers_paid_least = SELECT traveler_name, total_paid AS amount FROM $traveler_payments WHERE total_paid = $min_paid_amount;

    -- Travelers who pay most frequently
    LET $max_payment_count = math::max(SELECT VALUE payment_count FROM $traveler_payments);
    LET $travelers_pays_most_frequently = SELECT traveler_name, payment_count AS count FROM $traveler_payments WHERE payment_count = $max_payment_count;

    -- Travelers who pay least frequently
    LET $min_payment_count = math::min(SELECT VALUE payment_count FROM $traveler_payments);
    LET $travelers_pays_least_frequently = SELECT traveler_name, payment_count AS count FROM $traveler_payments WHERE payment_count = $min_payment_count;

    -- Major debtors/creditors
    LET $balances = SELECT * FROM fn::get_balances($chat);
    
    LET $debtors = SELECT debtor_name AS traveler_name, math::sum(debt) AS amount FROM $balances GROUP BY traveler_name;
    LET $max_debt = math::max(SELECT VALUE amount FROM $debtors);
    LET $major_debtors = SELECT * FROM $debtors WHERE amount = $max_debt;
    
    LET $creditors = SELECT creditor_name AS traveler_name, math::sum(debt) AS amount FROM $balances GROUP BY traveler_name;
    LET $max_credit = math::max(SELECT VALUE amount FROM $creditors);
    LET $major_creditors = SELECT * FROM $creditors WHERE amount = $max_credit;

    RETURN {
        travelers_count: $travelers_count,
        travelers_paid_most: $travelers_paid_most,
        travelers_paid_least: $travelers_paid_least,
        travelers_pays_most_frequently: $travelers_pays_most_frequently,
        travelers_pays_least_frequently: $travelers_pays_least_frequently,
        major_debtors: $major_debtors,
        major_creditors: $major_creditors,
    };
} COMMENT "Computes statistics for travelers within a chat, including who paid the most/least, who pays most/least frequently and who are the major debtors/creditors." PERMISSIONS FULL;

-- ------------------------------
-- TABLE: chat
-- ------------------------------

DEFINE TABLE OVERWRITE chat TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE currency ON chat TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE lang ON chat TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE last_interaction_utc ON chat TYPE datetime PERMISSIONS FOR select, create, update WHERE FULL;


-- ------------------------------
-- TABLE: expense
-- ------------------------------

DEFINE TABLE OVERWRITE expense TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE amount ON expense TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE chat ON expense TYPE record<chat> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE description ON expense TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE number ON expense TYPE int PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE timestamp_utc ON expense VALUE time::now() PERMISSIONS FOR select FULL, FOR create, update NONE;

DEFINE INDEX OVERWRITE expense_chat_number_index ON expense FIELDS chat, number UNIQUE;


-- ------------------------------
-- TABLE: owes
-- ------------------------------

DEFINE TABLE OVERWRITE owes TYPE RELATION IN traveler OUT traveler SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE amount ON owes TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE in ON owes TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON owes TYPE record<traveler> PERMISSIONS FULL;

DEFINE INDEX OVERWRITE owes_unique_relationships ON owes FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: paid_for
-- ------------------------------

DEFINE TABLE OVERWRITE paid_for TYPE RELATION IN traveler OUT expense SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE in ON paid_for TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON paid_for TYPE record<expense> PERMISSIONS FULL;

DEFINE INDEX OVERWRITE paid_for_unique_expense ON paid_for FIELDS out UNIQUE;


-- ------------------------------
-- TABLE: split
-- ------------------------------

DEFINE TABLE OVERWRITE split TYPE RELATION IN traveler OUT expense SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE amount ON split TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE in ON split TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON split TYPE record<expense> PERMISSIONS FULL;

DEFINE INDEX OVERWRITE split_unique_relationships ON split FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: transferred_to
-- ------------------------------

DEFINE TABLE OVERWRITE transferred_to TYPE RELATION IN traveler OUT traveler SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE amount ON transferred_to TYPE decimal PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE in ON transferred_to TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE number ON transferred_to TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON transferred_to TYPE record<traveler> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE timestamp_utc ON transferred_to VALUE time::now() PERMISSIONS FOR select FULL, FOR create, update NONE;

DEFINE INDEX OVERWRITE transferred_to_chat_number_index ON transferred_to FIELDS in.chat, number UNIQUE;


-- ------------------------------
-- TABLE: traveler
-- ------------------------------

DEFINE TABLE OVERWRITE traveler TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE chat ON traveler TYPE record<chat> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD OVERWRITE name ON traveler TYPE string ASSERT string::len($value) > 0 PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE INDEX OVERWRITE traveler_chat_name_index ON traveler FIELDS chat, name UNIQUE;
