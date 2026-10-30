## start

add-expense-start = The process can be interrupted at any time by sending <code>/{-cancel-command}</code>.
add-expense-ask-description = Set a description for this expense.

## receive_description

add-expense-ask-amount = How much is the expense?
add-expense-invalid-description = You sent an invalid text, please retry.

## receive_amount

add-expense-ask-paid-by = Who paid for this?
add-expense-invalid-amount = You sent an invalid amount, please retry.
add-expense-non-positive-amount = The amount must be greater than zero.

## receive_paid_by

add-expense-invalid-paid-by = You sent an invalid name, please retry.
add-expense-ask-shares = How would you like to split the expense? Type <code>/{-help-command} {-add-expense-command}</code> for more info.
add-expense-traveler-not-found = Couldn't find traveler <b>{$name}</b>. Specify the traveler who paid for this expense.
add-expense-traveler-generic-error = An error occured while looking for traveler <b>{$name}</b>. Please retry.

## start_split_among / receive_split_among

add-expense-continue-split = Continue splitting or type <code>{-end-kword}</code> to end the process.
add-expense-ok = <b>Expense recorded successfully!</b>
add-expense-error-on-computing-shares = An error occured while computing shares.
add-expense-creating-expense-generic-error = An error occured while creating expense.
add-expense-shares-parsing-error = An error occured while parsing the text. Please retry.
add-expense-invalid-shares = You sent an invalid text, please retry.
add-expense-shares-cleared = Previously entered shares have been cleared. Please retry.
