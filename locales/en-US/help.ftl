## Command

descr-command = These commands are supported:

## /help

descr-help = Show a help message for the specified command. If no command is specified, show the descriptions of all commands.
help-help = 
    /{-help-command} — {descr-help}

    Usage: /{-help-command} [command]

## /setlanguage

descr-set-language = Set the default language of the bot for this chat.
help-set-language =
    /{-set-language-command} — {descr-set-language}

    Available langauges:
    {$available-langs}

    Usage: /{-set-language-command} <language>

## /setcurrency

descr-set-currency = Set the default currency for this chat.
help-set-currency =
    /{-set-currency-command} — {descr-set-currency}

    Usage: /{-set-currency-command} <currency code>

## /addtraveler

descr-add-traveler = Add a traveler with the specified name to the travel plan.
help-add-traveler =
    /{-add-traveler-command} — {descr-add-traveler}

    Usage: /{-add-traveler-command} <name>

## /deletetraveler

descr-delete-traveler = Delete the traveler with the specified name from the travel plan.
help-delete-traveler =
    /{-delete-traveler-command} — {descr-delete-traveler}

    Usage: /{-delete-traveler-command} <name>

## /listtravelers

descr-list-travelers = Show the travelers in the travel plan.
help-list-travelers = 
    /{-list-travelers-command} — {descr-list-travelers}

    Usage: /{-list-travelers-command}

## /addexpense

descr-add-expense = Start a new interactive session to add an expense to the travel plan.
help-add-expense = 
    /{-add-expense-command} — {descr-add-expense}

    In the session, you will be asked to:
    - Send a message with the description of the expense.
    - Send a message with the amount of the expense.
    - Send a message with the name of the traveler who paid the expense.
    - Send a message with the travelers who partecipate in the expense and their shares.

    The process can be interrupted at any time by sending `/{-cancel-command}`. 

    To split the expense among multiple travelers you can:
    - Send a message for each traveler you want to share the expense with, or specify multiple travelers separating them by `{-split-among-entries-sep}`.
    - Use the format `<name>{-split-among-name-amount-sep} <amount>` where `<amount>` can be followed by `%` if it is a percentage of the residual amount.
    > Example: `Alice{-split-among-name-amount-sep} 50`, `Bob{-split-among-name-amount-sep} 20%`, `Charles`, `John{-split-among-name-amount-sep} 30{-split-among-entries-sep} Jane{-split-among-name-amount-sep} 10%` are all valid syntaxes.
    > Example: If the total is `100`, typing `Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-name-amount-sep} 60%` means that Alice will pay `40` so the residual is `60`, Bob will pay `24` (i.e. 40% of 60) and Charles will pay `36` (i.e. 60% of 60).

    - Omit `{-split-among-name-amount-sep} <amount>` if you want to evenly split the residual expense among the travelers.
    > Example: The input `Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-entries-sep} David` is equivalent to set both Charles and David amounts to 30%.

    - Enter `{-all-kword}` to split it evenly among all travelers.

    Usage: /{-add-expense-command}

## /deleteexpense

descr-delete-expense = Delete the expense with the specified identifying number from the travel plan.
help-delete-expense = 
    /{-delete-expense-command} — {descr-delete-expense}

    Usage: /{-delete-expense-command} <number>

## /listexpenses

descr-list-expenses = Show the expenses in the travel plan. If a description is specified, show only the expenses that match the given description. Supports fuzzy search for more flexible matching.
help-list-expenses = 
    /{-list-expenses-command} — {descr-list-expenses}

    Usage: /{-list-expenses-command} [description]

## /showexpense

descr-show-expense = Show the details of the expense with the specified identifying number.
help-show-expense = 
    /{-show-expense-command} — {descr-show-expense}

    Usage: /{-show-expense-command} <number>

## /transfer

descr-transfer = Transfer the specified amount from one traveler to another.
help-transfer = 
    /{-transfer-command} — {descr-transfer}

    Usage: /{-transfer-command} <sender> <receiver> <amount>

## /deletetransfer

descr-delete-transfer = Delete the transfer with the specified identifying number from the travel plan.
help-delete-transfer = 
    /{-delete-transfer-command} — {descr-delete-transfer}

    Usage: /{-delete-transfer-command} <number>

## /listtransfers

descr-list-transfers = Show the transfers in the travel plan. If a name is specified, show only the transfers that involve that traveler.
help-list-transfers = 
    /{-list-transfers-command} — {descr-list-transfers}

    Usage: /{-list-transfers-command} [name]

## /showbalances

descr-show-balances = Show the simplified balances of all travelers, minimizing the total number of transfers needed to settle the debts. If a name is specified, show the balance of the specified traveler.
help-show-balances = 
    /{-show-balances-command} — {descr-show-balances}

    Usage: /{-show-balances-command} [name]

## /cancel

descr-cancel = Cancel the currently running interactive command.
help-cancel = 
    /{-cancel-command} — {descr-cancel}

    Usage: /{-cancel-command}
