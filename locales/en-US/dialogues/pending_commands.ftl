## /add_traveler

add-traveler-ask-name = What's the name of the traveler? The process can be interrupted at any time by sending `/{-cancel-command}`.
add-traveler-invalid-name = You sent an invalid name, please retry.

## /delete_traveler

delete-traveler-ask-name = Which traveler do you want to delete? The process can be interrupted at any time by sending `/{-cancel-command}`.
delete-traveler-invalid-name = You sent an invalid name, please retry.
delete-traveler-confirm =
    Are you sure you want to delete traveler "{$name}"?
    ⚠️ All transfers involving this traveler will also be deleted.

## /delete_expense

delete-expense-ask-number = Which expense do you want to delete? Send the expense number. The process can be interrupted at any time by sending `/{-cancel-command}`.
delete-expense-invalid-number = You sent an invalid number, please retry.
delete-expense-confirm = Are you sure you want to delete expense #{$number}?

## /show_expense

show-expense-ask-number = Which expense do you want to show? Send the expense number. The process can be interrupted at any time by sending `/{-cancel-command}`.
show-expense-invalid-number = You sent an invalid number, please retry.

## /delete_transfer

delete-transfer-ask-number = Which transfer do you want to delete? Send the transfer number. The process can be interrupted at any time by sending `/{-cancel-command}`.
delete-transfer-invalid-number = You sent an invalid number, please retry.
delete-transfer-confirm = Are you sure you want to delete transfer #{$number}?

## /set_language

set-language-ask-langid = Which language do you want to set?
set-language-invalid-langid = You sent an invalid language identifier, please retry.

## /set_currency

set-currency-ask-currency = Which currency do you want to set? Select one from the list below or send the currency code if not present.
set-currency-invalid-currency = You sent an invalid currency code, please retry.

## /list_expenses

list-expenses-ask-description = Send a description (or part of it) to filter expenses by.

## /transfer

transfer-ask-from = Who is the sender? The process can be interrupted at any time by sending `/{-cancel-command}`.
transfer-ask-from-reprompt = You sent an invalid name, please retry. Who is the sender?
transfer-from-not-found = Traveler "{$name}" not found. Who is the sender?
transfer-ask-to = Who is the receiver of the transfer from {$name}?
transfer-ask-to-reprompt = You sent an invalid name, please retry. Who is the receiver?
transfer-to-not-found = Traveler "{$name}" not found. Who is the receiver?
transfer-ask-amount = How much did {$name} transfer?
transfer-invalid-amount = You sent an invalid amount, please retry.

## /cleartravelers

clear-travelers-confirm =
    Are you sure you want to delete all travelers?
    ⚠️ All transfers involving these travelers will also be deleted.
clear-travelers-show-expenses-prompt = Select a traveler to view their expenses, or tap "All" to view all.

## /clearexpenses

clear-expenses-confirm = Are you sure you want to delete all expenses?

## /cleartransfers

clear-transfers-confirm = Are you sure you want to delete all transfers?

## /clearall

clear-all-confirm =
    Are you sure you want to delete all travelers, expenses and transfers?
    ⚠️ This action cannot be undone.
