format-share-details = • <b>{$traveler-name}</b>: {$amount}
format-expense-details =
    <b>Number:</b> {$number} - <b>Description:</b> <i>{$description}</i>
    <b>Registered on:</b> {DATETIME($datetime, dateStyle: "long")}
    <b>Amount:</b> {$amount}
    <b>Paid by:</b> {$creditor}
    <b>Shares:</b>
    {$shares}
format-expense = [{DATETIME($datetime, dateStyle: "short")}] <b>Expense #{$number}</b>: <i>{$description}</i> - <b>{$amount}</b>
format-transfer = [{DATETIME($datetime, dateStyle: "short")}] <b>Transfer #{$number}</b>: <b>{$sender}</b> sent <b>{$amount}</b> to <b>{$receiver}</b>
format-traveler-stats-amount = ◦ <b>{$traveler-name}</b>: {$amount}
format-traveler-stats-frequency = 
    ◦ <b>{$traveler-name}</b>: {$count ->
        [one] {$count} expense
       *[other] {$count} expenses
    }
format-average-per-day = {$amount} per day [from {DATETIME($oldest-timestamp, dateStyle: "short")} to date {DATETIME($now, dateStyle: "short")}]
format-expense-stats =
    <b>Expense statistics:</b>
    { $count ->
        [0] • <b>Count:</b> {$count}
        *[other]
            • <b>Count:</b> {$count}
            • <b>Sum:</b> {$sum}
            • <b>Mean:</b> {$mean}
            • <b>Maximum expenses:</b> {$max}
            • <b>Minimum expenses:</b> {$min}
            • <b>Average per day:</b> {$average-per-day}
            • <b>Oldest:</b> {$oldest}
            • <b>Newest:</b> {$newest}
    }
format-transfer-stats =
    <b>Transfer statistics:</b>
    { $count ->
        [0] • <b>Count:</b> {$count}
        *[other]
            • <b>Count:</b> {$count}
            • <b>Sum:</b> {$sum}
            • <b>Mean:</b> {$mean}
            • <b>Maximum transfers:</b> {$max}
            • <b>Minimum transfers:</b> {$min}
            • <b>Average per day:</b> {$average-per-day}
            • <b>Oldest:</b> {$oldest}
            • <b>Newest:</b> {$newest}
    }
format-traveler-stats =
    <b>Traveler statistics:</b>
    { $count ->
        [0] • <b>Count:</b> {$count}
        *[other] 
            • <b>Count:</b> {$count}{ $expenses-count ->
                [0] {""}
                *[other]
                    {" "}
                    • <b>Who paid the most:</b> {$travelers-paid-most}
                    • <b>Who paid the least:</b> {$travelers-paid-least}
                    • <b>Who pays most frequently:</b> {$travelers-pays-most-frequently}
                    • <b>Who pays least frequently:</b> {$travelers-pays-least-frequently}
            }{ $balances-count ->
                [0] {""}
                *[other]
                    {" "}
                    • <b>Major creditors:</b> {$major-creditors}
                    • <b>Major debtors:</b> {$major-debtors}
            }
    }
format-stats =
    {$expense-stats}

    {$transfer-stats}

    {$traveler-stats}
