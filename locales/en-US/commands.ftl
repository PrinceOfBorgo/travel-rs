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
    Couldn't set the language. <code>{$langid}</code> is not available.

    <b>Available languages:</b>
    {$available-langs}
set-language-ok = Chat language correctly set to <b>{$language}</b>.

## /setcurrency

set-currency-ok = Chat default currency correctly set to <b>{$currency}</b>.
set-currency-not-available = <code>{$currency}</code> is not a recognized currency code. Please use a valid ISO 4217 code (e.g. <code>USD</code>, <code>EUR</code>) or a known crypto code (e.g. <code>BTC</code>, <code>ETH</code>).

## /addtraveler

add-traveler-already-added = Traveler <b>{$name}</b> has already been added to the travel plan.
add-traveler-ok = Traveler <b>{$name}</b> added successfully.

## /deletetraveler

delete-traveler-has-expenses =
    Traveler <b>{$name}</b> has paid for the following expenses:
    
    {$expenses}

    Please delete them first before deleting the traveler.
delete-traveler-not-found = Couldn't find traveler <b>{$name}</b> to delete.
delete-traveler-ok = Traveler <b>{$name}</b> deleted successfully.

## /listtravelers

list-travelers-not-found = No travelers found. Use <code>/{-add-traveler-command} &lt;name&gt;</code> to add one.

## /deleteexpense

delete-expense-not-found = Couldn't find expense <b>#{$number}</b> to delete.
delete-expense-ok = Expense <b>#{$number}</b> deleted successfully.

## /listexpenses

list-expenses-descr-not-found = No expenses match the specified description (~ <i>{$description}</i>).
list-expenses-not-found = No expenses found. Use <code>/{-add-expense-command}</code> to add one.

## /showexpense

show-expense-not-found = Couldn't find expense <b>#{$number}</b> to show the details.

## /transfer

transfer-ok = Transfer recorded successfully.
transfer-receiver-not-found = Couldn't find traveler <b>{$name}</b> to transfer money to.
transfer-sender-not-found = Couldn't find traveler <b>{$name}</b> to transfer money from.
transfer-same-sender-receiver = Sender and receiver cannot be the same traveler (<b>{$name}</b>).
transfer-non-positive-amount = The transfer amount must be greater than zero.

## /deletetransfer

delete-transfer-not-found = Couldn't find transfer <b>#{$number}</b> to delete.
delete-transfer-ok = Transfer <b>#{$number}</b> deleted successfully.

## /listtransfers

list-transfers-name-not-found = No transfers related to traveler <b>{$name}</b> found.
list-transfers-not-found = No transfers found. Use <code>/{-transfer-command} &lt;sender&gt; &lt;receiver&gt; &lt;amount&gt;</code> to add one.

## /showbalances

show-balances-ok = <b>{$debtor}</b> owes <b>{$debt}</b> to <b>{$creditor}</b>.
show-balances-settled-up = All travelers are settled up with everyone.
show-balances-traveler-ok = 
    <b>{$traveler-name}</b> { $traveler-is -> 
        *[debtor] owes <b>{$debt}</b> to
        [creditor] is owed <b>{$debt}</b> from
    } <b>{$other-traveler-name}</b>.
show-balances-traveler-settled-up = Traveler <b>{$name}</b> is settled up with everyone.
show-balances-traveler-not-found = Couldn't find traveler <b>{$name}</b> to show the balance.

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
    The following travelers have associated expenses and cannot be deleted:
    
    <b>{$travelers}</b>

    Please delete their expenses first, or use /{-clear-all-command} to clear everything.

## /clearall

clear-all-ok = All travelers, expenses and transfers cleared successfully.
clear-all-not-found = Nothing to clear.

## unknown command

invalid-command-usage = 
    Invalid usage of command: <code>{$command}</code>.

    {$help-message}
unknown-command = 
    Unknown command: <code>{$command}</code>.
unknown-command-best-match = 
    Unknown command: <code>{$command}</code>.
    Did you mean: {$best-match}?
