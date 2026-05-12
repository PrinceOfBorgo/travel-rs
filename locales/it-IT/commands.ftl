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
    Impossibile impostare la lingua. "{$langid}" non è disponibile.

    Lingue disponibili:
    {$available-langs}
set-language-ok = Lingua della chat impostata correttamente su {$language}.

## /setcurrency

set-currency-ok = Valuta predefinita della chat impostata correttamente su {$currency}.
set-currency-not-available = "{$currency}" non è un codice valuta riconosciuto. Usa un codice ISO 4217 valido (es. `USD`, `EUR`) o un codice crypto noto (es. `BTC`, `ETH`).

## /addtraveler

add-traveler-already-added = Il viaggiatore {$name} è già stato aggiunto al piano di viaggio.
add-traveler-ok = Viaggiatore {$name} aggiunto con successo.

## /deletetraveler

delete-traveler-has-expenses =
    Il viaggiatore {$name} ha pagato per le seguenti spese:
    
    {$expenses}

    Per favore, eliminale prima di eliminare il viaggiatore.
delete-traveler-not-found = Impossibile trovare il viaggiatore {$name} da eliminare.
delete-traveler-ok = Viaggiatore {$name} eliminato con successo.

## /listtravelers

list-travelers-not-found = Nessun viaggiatore trovato. Usa `/{-add-traveler-command} <name>` per aggiungerne uno.

## /deleteexpense

delete-expense-not-found = Impossibile trovare la spesa #{$number} da eliminare.
delete-expense-ok = Spesa #{$number} eliminata con successo.

## /listexpenses

list-expenses-descr-not-found = Nessuna spesa corrisponde alla descrizione specificata (~ "{$description}").
list-expenses-not-found = Nessuna spesa trovata. Usa `/{-add-expense-command}` per aggiungerne una.

## /showexpense

show-expense-not-found = Impossibile trovare la spesa #{$number} per mostrare i dettagli.

## /transfer

transfer-ok = Trasferimento registrato con successo.
transfer-receiver-not-found = Impossibile trovare il viaggiatore "{$name}" a cui trasferire denaro.
transfer-sender-not-found = Impossibile trovare il viaggiatore "{$name}" da cui trasferire denaro.
transfer-same-sender-receiver = Mittente e destinatario non possono essere lo stesso viaggiatore ("{$name}").
transfer-non-positive-amount = L'importo del trasferimento deve essere maggiore di zero.

## /deletetransfer

delete-transfer-not-found = Impossibile trovare il trasferimento #{$number} da eliminare.
delete-transfer-ok = Trasferimento #{$number} eliminato con successo.

## /listtransfers

list-transfers-name-not-found = Nessun trasferimento relativo al viaggiatore "{$name}" trovato.
list-transfers-not-found = Nessun trasferimento trovato. Usa `/{-transfer-command} <mittente> <destinatario> <importo>` per aggiungerne uno.

## /showbalances

show-balances-ok = {$debtor} deve {$debt} a {$creditor}.
show-balances-settled-up = Tutti i viaggiatori sono in pari con tutti.
show-balances-traveler-ok = 
    {$traveler-name} { $traveler-is -> 
        *[debtor] deve {$debt} a
        [creditor] deve ricevere {$debt} da
    } {$other-traveler-name}.
show-balances-traveler-settled-up = Il viaggiatore {$name} è in pari con tutti.
show-balances-traveler-not-found = Impossibile trovare il viaggiatore "{$name}" per mostrare il saldo.

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
    I seguenti viaggiatori hanno spese associate e non possono essere eliminati: {$travelers}.
    Elimina prima le loro spese, oppure usa /{-clear-all-command} per eliminare tutto.

## /clearall

clear-all-ok = Tutti i viaggiatori, spese e trasferimenti eliminati con successo.
clear-all-not-found = Niente da eliminare.

## unknown command

invalid-command-usage = 
    Uso non valido del comando: {$command}.

    {$help-message}
unknown-command = 
    Comando sconosciuto: {$command}.
unknown-command-best-match = 
    Comando sconosciuto: {$command}.
    Intendevi: {$best-match}?
