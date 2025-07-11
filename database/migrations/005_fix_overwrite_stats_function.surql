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
    LET $diff_days = (time::unix($current_timestamp) - time::unix($oldest_timestamp)) / 60 / 60 / 24 + 1;
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
    LET $diff_days = (time::unix($current_timestamp) - time::unix($oldest_timestamp)) / 60 / 60 / 24 + 1;
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
