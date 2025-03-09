## start

add-expense-start = The process can be interrupted at any time by sending `/{-cancel-command}`.
add-expense-ask-description = How would you describe this expense?

## receive_description

add-expense-ask-amount = How much is the expense?
add-expense-invalid-description = You sent an invalid text, please retry.

## receive_amount

add-expense-ask-paid-by = Who paid for this?
add-expense-invalid-amount = You sent an invalid amount, please retry.

## receive_paid_by

add-expense-invalid-paid-by = You sent an invalid name, please retry.
add-expense-ask-shares = How would you like to split the expense? Type `/{-help-command} {-add-expense-command}` for more info.
add-expense-traveler-not-found = Couldn't find traveler {$name}. Specify the traveler who paid for this expense.
add-expense-traveler-generic-error = An error occured while looking for traveler {$name}. Please retry.

## start_split_among / receive_split_among

add-expense-continue-split = Continue splitting or type `{-end-kword}` to end the process.
add-expense-ok = Expense added successfully!
add-expense-error-on-computing-shares = An error occured while computing shares.
add-expense-creating-expense-generic-error = An error occured while creating expense.
add-expense-shares-parsing-error = An error occured while parsing the text. Please retry.
add-expense-invalid-shares = You sent an invalid text, please retry.
add-expense-shares-cleared = Previously entered shares have been cleared. Please retry.