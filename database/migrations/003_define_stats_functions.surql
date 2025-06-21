DEFINE FUNCTION fn::expense_stats($chat: record<chat>) {
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

DEFINE FUNCTION fn::transfer_stats($chat: record<chat>) {
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

DEFINE FUNCTION fn::traveler_stats($chat: record<chat>) {
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
