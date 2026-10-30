## Command

descr-command = <b>These commands are supported:</b>

## /help

descr-help = Show a help message for the specified command. If no command is specified, show the descriptions of all commands.
help-help = 
    <b>/{-help-command}</b> — {descr-help}

    <b>Usage:</b> <code>/{-help-command} [command]</code>

## /setlanguage

descr-set-language = Set the default language of the bot for this chat.
help-set-language =
    <b>/{-set-language-command}</b> — {descr-set-language}

    <b>Available languages:</b>
    {$available-langs}

    <b>Usage:</b> <code>/{-set-language-command} &lt;language&gt;</code>

## /setcurrency

descr-set-currency = Set the default currency for the travel plan.
help-set-currency =
    <b>/{-set-currency-command}</b> — {descr-set-currency}

    <b>Usage:</b> <code>/{-set-currency-command} &lt;currency code&gt;</code>

## /addtraveler

descr-add-traveler = Add a traveler with the specified name to the travel plan.
help-add-traveler =
    <b>/{-add-traveler-command}</b> — {descr-add-traveler}

    <b>Usage:</b> <code>/{-add-traveler-command} &lt;name&gt;</code>

## /deletetraveler

descr-delete-traveler = Delete the traveler with the specified name (and associated transfers) from the travel plan.
help-delete-traveler =
    <b>/{-delete-traveler-command}</b> — {descr-delete-traveler}

    <b>Usage:</b> <code>/{-delete-traveler-command} &lt;name&gt;</code>

## /listtravelers

descr-list-travelers = Show the travelers in the travel plan.
help-list-travelers = 
    <b>/{-list-travelers-command}</b> — {descr-list-travelers}

    <b>Usage:</b> <code>/{-list-travelers-command}</code>

## /addexpense

descr-add-expense = Start a new interactive session to add an expense to the travel plan.
help-add-expense = 
    <b>/{-add-expense-command}</b> — {descr-add-expense}

    In the session, you will be asked to:
    1. Send a message with the description of the expense.
    2. Send a message with the amount of the expense.
    3. Send a message with the name of the traveler who paid the expense.
    4. Send one or more messages with the travelers who partecipate in the expense and their shares.

    The process can be interrupted at any time by sending <code>/{-cancel-command}</code>.

    To split the expense among multiple travelers you can:
    • Send a message for each traveler you want to share the expense with, or specify multiple travelers separating them by <code>{-split-among-entries-sep}</code>.
    • Use the format <code>&lt;name&gt;{-split-among-name-amount-sep} &lt;amount&gt;</code> where <code>&lt;amount&gt;</code> can be followed by <code>%</code> if it is a percentage of the residual amount.
    <blockquote><i>Example:</i> <code>Alice{-split-among-name-amount-sep} 50</code>, <code>Bob{-split-among-name-amount-sep} 20%</code>, <code>Charles</code>, <code>John{-split-among-name-amount-sep} 30{-split-among-entries-sep} Jane{-split-among-name-amount-sep} 10%</code> are all valid syntaxes.
    <i>Example:</i> If the total is <code>100</code>, typing <code>Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-name-amount-sep} 60%</code> means that Alice will pay <code>40</code> so the residual is <code>60</code>, Bob will pay <code>24</code> (i.e. 40% of 60) and Charles will pay <code>36</code> (i.e. 60% of 60).</blockquote>

    • Omit <code>{-split-among-name-amount-sep} &lt;amount&gt;</code> if you want to evenly split the residual expense among the travelers.
    <blockquote><i>Example:</i> The input <code>Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-entries-sep} David</code> is equivalent to set both Charles and David amounts to 30%.</blockquote>

    • Enter <code>{-all-kword}</code> to split it evenly among all travelers.

    <b>Usage:</b> <code>/{-add-expense-command}</code>

## /deleteexpense

descr-delete-expense = Delete the expense with the specified identifying number from the travel plan.
help-delete-expense = 
    <b>/{-delete-expense-command}</b> — {descr-delete-expense}

    <b>Usage:</b> <code>/{-delete-expense-command} &lt;number&gt;</code>

## /listexpenses

descr-list-expenses = Show the expenses in the travel plan. If a description is specified, show only the expenses that match the given description. Supports fuzzy search for more flexible matching.
help-list-expenses = 
    <b>/{-list-expenses-command}</b> — {descr-list-expenses}

    <b>Usage:</b> <code>/{-list-expenses-command} [description]</code>

## /showexpense

descr-show-expense = Show the details of the expense with the specified identifying number.
help-show-expense = 
    <b>/{-show-expense-command}</b> — {descr-show-expense}

    <b>Usage:</b> <code>/{-show-expense-command} &lt;number&gt;</code>

## /transfer

descr-transfer = Transfer the specified amount from one traveler to another.
help-transfer = 
    <b>/{-transfer-command}</b> — {descr-transfer}

    <b>Usage:</b> <code>/{-transfer-command} &lt;sender&gt; &lt;receiver&gt; &lt;amount&gt;</code>

## /deletetransfer

descr-delete-transfer = Delete the transfer with the specified identifying number from the travel plan.
help-delete-transfer = 
    <b>/{-delete-transfer-command}</b> — {descr-delete-transfer}

    <b>Usage:</b> <code>/{-delete-transfer-command} &lt;number&gt;</code>

## /listtransfers

descr-list-transfers = Show the transfers in the travel plan. If a name is specified, show only the transfers that involve that traveler.
help-list-transfers = 
    <b>/{-list-transfers-command}</b> — {descr-list-transfers}

    <b>Usage:</b> <code>/{-list-transfers-command} [name]</code>

## /showbalances

descr-show-balances = Show the simplified balances of all travelers, minimizing the total number of transfers needed to settle the debts. If a name is specified, show the balance of the specified traveler.
help-show-balances = 
    <b>/{-show-balances-command}</b> — {descr-show-balances}

    <b>Usage:</b> <code>/{-show-balances-command} [name]</code>

## /showstats

descr-show-stats = Show the statistics for expenses, transfers and travelers in the travel plan.
help-show-stats = 
    <b>/{-show-stats-command}</b> — {descr-show-stats}

    <b>Usage:</b> <code>/{-show-stats-command}</code>

## /cleartravelers

descr-clear-travelers = Delete all travelers and their associated transfers from the travel plan.
help-clear-travelers =
    <b>/{-clear-travelers-command}</b> — {descr-clear-travelers}

    <b>Usage:</b> <code>/{-clear-travelers-command}</code>

## /clearexpenses

descr-clear-expenses = Delete all expenses from the travel plan.
help-clear-expenses =
    <b>/{-clear-expenses-command}</b> — {descr-clear-expenses}

    <b>Usage:</b> <code>/{-clear-expenses-command}</code>

## /cleartransfers

descr-clear-transfers = Delete all transfers from the travel plan.
help-clear-transfers =
    <b>/{-clear-transfers-command}</b> — {descr-clear-transfers}

    <b>Usage:</b> <code>/{-clear-transfers-command}</code>

## /clearall

descr-clear-all = Delete all travelers, expenses and transfers from the travel plan.
help-clear-all =
    <b>/{-clear-all-command}</b> — {descr-clear-all}

    <b>Usage:</b> <code>/{-clear-all-command}</code>

## /cancel

descr-cancel = Cancel the currently running interactive command.
help-cancel = 
    <b>/{-cancel-command}</b> — {descr-cancel}

    <b>Usage:</b> <code>/{-cancel-command}</code>
