format-share-details = - {$traveler-name}: {$amount}
format-expense-details =
    Number: {$number} - Description: {$description}
    Registered on: {DATETIME($datetime, dateStyle: "long")}
    Amount: {$amount}
    Paid by: {$creditor}
    Shares:
    {$shares}
format-expense = [{DATETIME($datetime, dateStyle: "short")}] Expense #{$number}: {$description} - {$amount}
format-transfer = [{DATETIME($datetime, dateStyle: "short")}] Transfer #{$number}: {$sender} sent {$amount} to {$receiver}
format-traveler-stats-amount = {$traveler-name}: {$amount}
format-traveler-stats-frequency = 
    {$traveler-name}: {$count ->
        [one] {$count} expense
       *[other] {$count} expenses
    }
format-average-per-day = {$amount} per day [from {DATETIME($oldest-timestamp, dateStyle: "short")} to date {DATETIME($now, dateStyle: "short")}]
format-expense-stats =
    Expense statistics:
    { $count ->
        [0] - Count: {$count}
        *[other]
            - Count: {$count}
            - Sum: {$sum}
            - Mean: {$mean}
            - Maximum expenses: {$max}
            - Minimum expenses: {$min}
            - Average per day: {$average-per-day}
            - Oldest: {$oldest}
            - Newest: {$newest}
    }
format-transfer-stats =
    Transfer statistics:
    { $count ->
        [0] - Count: {$count}
        *[other]
            - Count: {$count}
            - Sum: {$sum}
            - Mean: {$mean}
            - Maximum transfers: {$max}
            - Minimum transfers: {$min}
            - Average per day: {$average-per-day}
            - Oldest: {$oldest}
            - Newest: {$newest}
    }
format-traveler-stats =
    Traveler statistics:
    { $count ->
        [0] - Count: {$count}
        *[other] 
            - Count: {$count}{ $expenses-count ->
                [0] {""}
                *[other]
                    {" "}
                    - Who paid the most: {$travelers-paid-most}
                    - Who paid the least: {$travelers-paid-least}
                    - Who pays most frequently: {$travelers-pays-most-frequently}
                    - Who pays least frequently: {$travelers-pays-least-frequently}
            }{ $balances-count ->
                [0] {""}
                *[other]
                    {" "}
                    - Major creditors: {$major-creditors}
                    - Major debtors: {$major-debtors}
            }
    }
format-stats =
    {$expense-stats}

    {$transfer-stats}

    {$traveler-stats}
