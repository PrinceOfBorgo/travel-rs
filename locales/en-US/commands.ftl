## General

i18n-process-already-running = Another process is already running, please cancel it first sending /{-i18n-cancel-command}.

## /addtraveler

i18n-add-traveler-already-added = Traveler {$name} has already been added to the travel plan.
i18n-add-traveler-ok = Traveler {$name} added successfully.

## /cancel

i18n-cancel-no-process-to-cancel = There is no process to cancel.
i18n-cancel-ok = The process was cancelled.

## /deleteexpense

i18n-delete-expense-not-found = Couldn't find expense #{$number} to delete.
i18n-delete-expense-ok = Expense #{$number} deleted successfully.

## /deletetraveler

i18n-delete-traveler-not-found = Couldn't find traveler {$name} to delete.
i18n-delete-traveler-ok = Traveler {$name} deleted successfully.

## /findexpenses

i18n-find-expenses-not-found = No expenses match the specified description (~ \"{$description}\").

## /listexpenses

i18n-list-expenses-not-found = No expenses found. Use `/{-i18n-addexpense-command}` to add one.

## /listtravelers

i18n-list-travelers-not-found = No travelers found. Use `/{-i18n-addtraveler-command} <name>` to add one.

## /showbalance

i18n-show-balance-ok = 
    {$traveler-name} { $traveler-is -> 
        *[debtor] owes {$debt} to
        [creditor] is owed {$debt} from
    } {$other-traveler-name}.
i18n-show-balance-settled-up = Traveler {$name} is settled up with everyone.
i18n-show-balance-traveler-not-found = Couldn't find traveler {$name} to show the balance.

## /showbalances

i18n-show-balances-settled-up = All travelers are settled up with everyone.

## /showexpense

i18n-show-expense-not-found = Couldn't find expense #{$number} to show the details.
i18n-show-expense-ok = 
    Number: {$number} - Description: {$description}
    Amount: {$amount}
    Paid by: {$creditor}
    Split among:
    {$shares}

## /transfer

i18n-transfer-ok = Transfer recorded successfully.
i18n-transfer-sender-not-found = Couldn't find traveler \"{$name}\" to transfer money from.
i18n-transfer-receiver-not-found = Couldn't find traveler \"{$name}\" to transfer money to.

## unknown command

i18n-invalid-command-usage = 
    Invalid usage of command: {$command}.

    {$help_message}
i18n-unknown-command = 
    Unknown command: {$command}.
i18n-unknown-command-best-match = 
    Unknown command: {$command}.
    Did you mean: /{$best-match}?
