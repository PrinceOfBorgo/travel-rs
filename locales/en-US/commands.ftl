## general

process-already-running = Another process is already running, please cancel it first sending /{-cancel-command}.

## /addtraveler

add-traveler-already-added = Traveler {$name} has already been added to the travel plan.
add-traveler-ok = Traveler {$name} added successfully.

## /cancel

cancel-no-process-to-cancel = There is no process to cancel.
cancel-ok = The process was cancelled.

## /deleteexpense

delete-expense-not-found = Couldn't find expense #{$number} to delete.
delete-expense-ok = Expense #{$number} deleted successfully.

## /deletetraveler

delete-traveler-not-found = Couldn't find traveler {$name} to delete.
delete-traveler-ok = Traveler {$name} deleted successfully.

## /findexpenses

find-expenses-not-found = No expenses match the specified description (~ "{$description}").

## /listexpenses

list-expenses-not-found = No expenses found. Use `/{-add-expense-command}` to add one.

## /listtravelers

list-travelers-not-found = No travelers found. Use `/{-add-traveler-command} <name>` to add one.

## /showbalance

show-balance-ok = 
    {$traveler-name} { $traveler-is -> 
        *[debtor] owes {$debt} to
        [creditor] is owed {$debt} from
    } {$other-traveler-name}.
show-balance-settled-up = Traveler {$name} is settled up with everyone.
show-balance-traveler-not-found = Couldn't find traveler {$name} to show the balance.

## /showbalances

show-balances-ok = {$debtor} owes {$debt} to {$creditor}.
show-balances-settled-up = All travelers are settled up with everyone.

## /showexpense

show-expense-not-found = Couldn't find expense #{$number} to show the details.
show-expense-ok = 
    Number: {$number} - Description: {$description}
    Amount: {$amount}
    Paid by: {$creditor}
    Split among:
    {$shares}

## /transfer

transfer-ok = Transfer recorded successfully.
transfer-receiver-not-found = Couldn't find traveler "{$name}" to transfer money to.
transfer-sender-not-found = Couldn't find traveler "{$name}" to transfer money from.

## unknown command

invalid-command-usage = 
    Invalid usage of command: {$command}.

    {$help-message}
unknown-command = 
    Unknown command: {$command}.
unknown-command-best-match = 
    Unknown command: {$command}.
    Did you mean: /{$best-match}?
