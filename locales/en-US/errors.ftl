## CommandError

command-error-empty-input = No input provided.
command-error-help = No help available for command /{$command}.
command-error-set-language = Couldn't set language "{$langid}".
command-error-set-currency = Couldn't set currency "{$currency}".
command-error-add-traveler = Couldn't add traveler named "{$name}".
command-error-delete-traveler = Couldn't delete traveler named "{$name}".
command-error-list-travelers = Couldn't list travelers.
command-error-delete-expense = Couldn't delete expense #{$number}.
command-error-list-expenses = Couldn't list expenses.
command-error-show-expense = Couldn't show expense #{$number}.
command-error-transfer = Couldn't transfer {$amount} from traveler "{$sender}" to "{$receiver}".
command-error-delete-transfer = Couldn't delete transfer #{$number}.
command-error-list-transfers = Couldn't list transfers.
command-error-show-balance = Couldn't show balance for traveler "{$name}".
command-error-show-balances = Couldn't show balances.

## NameValidationError

name-validation-error-starts-with-slash = The name "{$name}" starts with a slash "/".
name-validation-error-invalid-char = The name "{$name}" contains an invalid character: "{$char}".
name-validation-error-reserved-keyword = "{$name}" is a reserved keyword.

## AddExpenseError

add-expense-error-repeated-traveler-name = Traveler "{$name}" has already been added to the expense.
add-expense-error-traveler-not-found = Cannot find traveler "{$name}" in the current travel plan.
add-expense-error-expense-too-high = The expenses assigned to travelers exceed the total amount: {$amount}.
add-expense-error-expense-too-low = The expense ({$expense}) is less than the total amount: {$amount}.
add-expense-error-invalid-format = Invalid format: "{$input}"
add-expense-error-no-travelers-specified = No travelers have been specified.

## EndError

end-error-closing-dialogue = An error occured while closing the process.
end-error-no-expense-created = No expense has been created.
