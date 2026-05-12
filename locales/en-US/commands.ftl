## general

process-already-running = Another process ({$process}) is already running, please cancel it first sending /{-cancel-command}.
running-process-unknown = unknown
running-process-add-expense = /{-add-expense-command}
running-process-add-traveler = /{-add-traveler-command}
running-process-delete-traveler = /{-delete-traveler-command}
running-process-delete-expense = /{-delete-expense-command}
running-process-show-expense = /{-show-expense-command}
running-process-delete-transfer = /{-delete-transfer-command}
running-process-set-language = /{-set-language-command}
running-process-set-currency = /{-set-currency-command}
running-process-list-expenses = /{-list-expenses-command}
running-process-transfer = /{-transfer-command}
running-process-clear-travelers = /{-clear-travelers-command}
running-process-clear-expenses = /{-clear-expenses-command}
running-process-clear-transfers = /{-clear-transfers-command}
running-process-clear-all = /{-clear-all-command}

## /setlanguage

set-language-not-available =
    Couldn't set the language. "{$langid}" is not available.

    Available languages:
    {$available-langs}
set-language-ok = Chat language correctly set to {$language}.

## /setcurrency

set-currency-ok = Chat default currency correctly set to {$currency}.
set-currency-not-available = "{$currency}" is not a recognized currency code. Please use a valid ISO 4217 code (e.g. `USD`, `EUR`) or a known crypto code (e.g. `BTC`, `ETH`).

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
transfer-same-sender-receiver = Sender and receiver cannot be the same traveler ("{$name}").
transfer-non-positive-amount = The transfer amount must be greater than zero.

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
cancel-ok = The process ({$process}) was cancelled.

## /clearexpenses

clear-expenses-ok = All expenses cleared successfully.
clear-expenses-not-found = No expenses to clear.

## /cleartransfers

clear-transfers-ok = All transfers cleared successfully.
clear-transfers-not-found = No transfers to clear.

## /cleartravelers

clear-travelers-ok = All travelers cleared successfully.
clear-travelers-not-found = No travelers to clear.
clear-travelers-has-expenses =
    The following travelers have associated expenses and cannot be deleted: {$travelers}.
    Please delete their expenses first, or use /{-clear-all-command} to clear everything.

## /clearall

clear-all-ok = All travelers, expenses and transfers cleared successfully.
clear-all-not-found = Nothing to clear.

## unknown command

invalid-command-usage = 
    Invalid usage of command: {$command}.

    {$help-message}
unknown-command = 
    Unknown command: {$command}.
unknown-command-best-match = 
    Unknown command: {$command}.
    Did you mean: {$best-match}?
