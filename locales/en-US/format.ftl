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
