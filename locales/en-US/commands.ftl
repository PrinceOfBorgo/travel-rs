## general

process-already-running = Another process is already running, please cancel it first sending /{-cancel-command}.

## /setlanguage

set-language-not-available =
    Couldn't set the language. "{$langid}" is not available.

    Available langauges:
    {$available-langs}
set-language-ok = Chat language correctly set to {$langid}.

## /setcurrency

set-currency-ok = Chat default currency correctly set to {$currency}.

## /addtraveler

add-traveler-already-added = Traveler {$name} has already been added to the travel plan.
add-traveler-ok = Traveler {$name} added successfully.

## /deletetraveler

delete-traveler-has-expenses =
    Traveler {$name} has paid for the following expenses:
    
    {$expenses}

    Please delete them first before deleting the traveler.
delete-traveler-not-found = Couldn't find traveler {$name} to delete.
delete-traveler-ok = Traveler {$name} deleted successfully.

## /listtravelers

list-travelers-not-found = No travelers found. Use `/{-add-traveler-command} <name>` to add one.

## /deleteexpense

delete-expense-not-found = Couldn't find expense #{$number} to delete.
delete-expense-ok = Expense #{$number} deleted successfully.

## /listexpenses

list-expenses-descr-not-found = No expenses match the specified description (~ "{$description}").
list-expenses-not-found = No expenses found. Use `/{-add-expense-command}` to add one.

## /showexpense

show-expense-not-found = Couldn't find expense #{$number} to show the details.

## /transfer

transfer-ok = Transfer recorded successfully.
transfer-receiver-not-found = Couldn't find traveler "{$name}" to transfer money to.
transfer-sender-not-found = Couldn't find traveler "{$name}" to transfer money from.

## /deletetransfer

delete-transfer-not-found = Couldn't find transfer #{$number} to delete.
delete-transfer-ok = Transfer #{$number} deleted successfully.

## /listtransfers

list-transfers-name-not-found = No transfers related to traveler "{$name}" found.
list-transfers-not-found = No transfers found. Use `/{-transfer-command} <sender> <receiver> <amount>` to add one.

## /showbalances

show-balances-ok = {$debtor} owes {$debt} to {$creditor}.
show-balances-settled-up = All travelers are settled up with everyone.
show-balances-traveler-ok = 
    {$traveler-name} { $traveler-is -> 
        *[debtor] owes {$debt} to
        [creditor] is owed {$debt} from
    } {$other-traveler-name}.
show-balances-traveler-settled-up = Traveler {$name} is settled up with everyone.
show-balances-traveler-not-found = Couldn't find traveler "{$name}" to show the balance.

## /cancel

cancel-no-process-to-cancel = There is no process to cancel.
cancel-ok = The process was cancelled.

## unknown command

invalid-command-usage = 
    Invalid usage of command: {$command}.

    {$help-message}
unknown-command = 
    Unknown command: {$command}.
unknown-command-best-match = 
    Unknown command: {$command}.
    Did you mean: {$best-match}?
