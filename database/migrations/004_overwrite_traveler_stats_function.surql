DEFINE FUNCTION OVERWRITE fn::traveler_stats($chat: record<chat>) {
    LET $expenses = SELECT amount, (<-paid_for<-traveler)[0] AS creditor FROM expense WHERE chat = $chat;
    LET $balances = SELECT * FROM fn::get_balances($chat);

    LET $travelers_count = array::len(SELECT * FROM traveler WHERE chat = $chat);
    LET $expenses_count = array::len($expenses);
    LET $balances_count = array::len($balances);

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
    LET $debtors = SELECT debtor_name AS traveler_name, math::sum(debt) AS amount FROM $balances GROUP BY traveler_name;
    LET $max_debt = math::max(SELECT VALUE amount FROM $debtors);
    LET $major_debtors = SELECT * FROM $debtors WHERE amount = $max_debt;
    
    LET $creditors = SELECT creditor_name AS traveler_name, math::sum(debt) AS amount FROM $balances GROUP BY traveler_name;
    LET $max_credit = math::max(SELECT VALUE amount FROM $creditors);
    LET $major_creditors = SELECT * FROM $creditors WHERE amount = $max_credit;

    RETURN {
        travelers_count: $travelers_count,
        expenses_count: $expenses_count,
        balances_count: $balances_count,
        travelers_paid_most: $travelers_paid_most,
        travelers_paid_least: $travelers_paid_least,
        travelers_pays_most_frequently: $travelers_pays_most_frequently,
        travelers_pays_least_frequently: $travelers_pays_least_frequently,
        major_debtors: $major_debtors,
        major_creditors: $major_creditors,
    };
} COMMENT "Computes statistics for travelers within a chat, including who paid the most/least, who pays most/least frequently and who are the major debtors/creditors." PERMISSIONS FULL;
