## general

process-already-running = Un altro processo ({$process}) è già in esecuzione, per favore annullalo prima inviando /{-cancel-command}.
running-process-unknown = sconosciuto
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
    Impossibile impostare la lingua. <code>{$langid}</code> non è disponibile.

    <b>Lingue disponibili:</b>
    {$available-langs}
set-language-ok = Lingua della chat impostata correttamente su <b>{$language}</b>.

## /setcurrency

set-currency-ok = Valuta predefinita della chat impostata correttamente su <b>{$currency}</b>.
set-currency-not-available = <code>{$currency}</code> non è un codice valuta riconosciuto. Usa un codice ISO 4217 valido (es. <code>USD</code>, <code>EUR</code>) o un codice crypto noto (es. <code>BTC</code>, <code>ETH</code>).

## /addtraveler

add-traveler-already-added = Il viaggiatore <b>{$name}</b> è già stato aggiunto al piano di viaggio.
add-traveler-ok = Viaggiatore <b>{$name}</b> aggiunto con successo.

## /deletetraveler

delete-traveler-has-expenses =
    Il viaggiatore <b>{$name}</b> ha pagato per le seguenti spese:
    
    {$expenses}

    Eliminale prima di eliminare il viaggiatore.
delete-traveler-not-found = Impossibile trovare il viaggiatore <b>{$name}</b> da eliminare.
delete-traveler-ok = Viaggiatore <b>{$name}</b> eliminato con successo.

## /listtravelers

list-travelers-not-found = Nessun viaggiatore trovato. Usa <code>/{-add-traveler-command} &lt;nome&gt;</code> per aggiungerne uno.

## /deleteexpense

delete-expense-not-found = Impossibile trovare la spesa <b>#{$number}</b> da eliminare.
delete-expense-ok = Spesa <b>#{$number}</b> eliminata con successo.

## /listexpenses

list-expenses-descr-not-found = Nessuna spesa corrisponde alla descrizione specificata (~ <i>{$description}</i>).
list-expenses-not-found = Nessuna spesa trovata. Usa <code>/{-add-expense-command}</code> per aggiungerne una.

## /showexpense

show-expense-not-found = Impossibile trovare la spesa <b>#{$number}</b> per mostrare i dettagli.

## /transfer

transfer-ok = Trasferimento registrato con successo.
transfer-receiver-not-found = Impossibile trovare il viaggiatore <b>{$name}</b> a cui trasferire denaro.
transfer-sender-not-found = Impossibile trovare il viaggiatore <b>{$name}</b> da cui trasferire denaro.
transfer-same-sender-receiver = Mittente e destinatario non possono essere lo stesso viaggiatore (<b>{$name}</b>).
transfer-non-positive-amount = L'importo del trasferimento deve essere maggiore di zero.

## /deletetransfer

delete-transfer-not-found = Impossibile trovare il trasferimento <b>#{$number}</b> da eliminare.
delete-transfer-ok = Trasferimento <b>#{$number}</b> eliminato con successo.

## /listtransfers

list-transfers-name-not-found = Nessun trasferimento relativo al viaggiatore <b>{$name}</b> trovato.
list-transfers-not-found = Nessun trasferimento trovato. Usa <code>/{-transfer-command} &lt;mittente&gt; &lt;destinatario&gt; &lt;importo&gt;</code> per aggiungerne uno.

## /showbalances

show-balances-ok = <b>{$debtor}</b> deve <b>{$debt}</b> a <b>{$creditor}</b>.
show-balances-settled-up = Tutti i viaggiatori sono in pari con tutti.
show-balances-traveler-ok = 
    <b>{$traveler-name}</b> { $traveler-is -> 
        *[debtor] deve <b>{$debt}</b> a
        [creditor] deve ricevere <b>{$debt}</b> da
    } <b>{$other-traveler-name}</b>.
show-balances-traveler-settled-up = Il viaggiatore <b>{$name}</b> è in pari con tutti.
show-balances-traveler-not-found = Impossibile trovare il viaggiatore <b>{$name}</b> per mostrare il saldo.

## /cancel

cancel-no-process-to-cancel = Non c'è nessun processo da annullare.
cancel-ok = Il processo ({$process}) è stato annullato.

## /clearexpenses

clear-expenses-ok = Tutte le spese eliminate con successo.
clear-expenses-not-found = Nessuna spesa da eliminare.

## /cleartransfers

clear-transfers-ok = Tutti i trasferimenti eliminati con successo.
clear-transfers-not-found = Nessun trasferimento da eliminare.

## /cleartravelers

clear-travelers-ok = Tutti i viaggiatori eliminati con successo.
clear-travelers-not-found = Nessun viaggiatore da eliminare.
clear-travelers-has-expenses =
    I seguenti viaggiatori hanno spese associate e non possono essere eliminati:

    <b>{$travelers}</b>

    Elimina prima le loro spese, oppure usa /{-clear-all-command} per eliminare tutto.

## /clearall

clear-all-ok = Tutti i viaggiatori, spese e trasferimenti eliminati con successo.
clear-all-not-found = Niente da eliminare.

## unknown command

invalid-command-usage = 
    Uso non valido del comando: <code>{$command}</code>.

    {$help-message}
unknown-command = 
    Comando sconosciuto: <code>{$command}</code>.
unknown-command-best-match = 
    Comando sconosciuto: <code>{$command}</code>.
    Intendevi: {$best-match}?
