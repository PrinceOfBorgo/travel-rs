## general

process-already-running = Un altro processo è già in esecuzione, per favore annullalo prima inviando /{-cancel-command}.

## /setlanguage

set-language-not-available =
    Impossibile impostare la lingua. "{$langid}" non è disponibile.

    Lingue disponibili:
    {$available-langs}
set-language-ok = Lingua della chat impostata correttamente su {$langid}.

## /setcurrency

set-currency-ok = Valuta predefinita della chat impostata correttamente su {$currency}.

## /addtraveler

add-traveler-already-added = Il viaggiatore {$name} è già stato aggiunto al piano di viaggio.
add-traveler-ok = Viaggiatore {$name} aggiunto con successo.

## /cancel

cancel-no-process-to-cancel = Non c'è nessun processo da annullare.
cancel-ok = Il processo è stato annullato.

## /deleteexpense

delete-expense-not-found = Impossibile trovare la spesa #{$number} da eliminare.
delete-expense-ok = Spesa #{$number} eliminata con successo.

## /deletetraveler

delete-traveler-has-expenses =
    Il viaggiatore {$name} ha pagato per le seguenti spese:
    
    {$expenses}

    Per favore, eliminale prima di eliminare il viaggiatore.
delete-traveler-not-found = Impossibile trovare il viaggiatore {$name} da eliminare.
delete-traveler-ok = Viaggiatore {$name} eliminato con successo.

## /listexpenses

list-expenses-descr-not-found = Nessuna spesa corrisponde alla descrizione specificata (~ "{$description}").
list-expenses-not-found = Nessuna spesa trovata. Usa `/{-add-expense-command}` per aggiungerne una.

## /listtravelers

list-travelers-not-found = Nessun viaggiatore trovato. Usa `/{-add-traveler-command} <name>` per aggiungerne uno.

## /showbalances

show-balances-ok = {$debtor} deve {$debt} a {$creditor}.
show-balances-settled-up = Tutti i viaggiatori sono in pari con tutti.
show-balances-traveler-ok = 
    {$traveler-name} { $traveler-is -> 
        *[debtor] deve {$debt} a
        [creditor] deve riceve {$debt} da
    } {$other-traveler-name}.
show-balances-traveler-settled-up = Il viaggiatore {$name} è in pari con tutti.
show-balances-traveler-not-found = Impossibile trovare il viaggiatore "{$name}" per mostrare il saldo.

## /showexpense

show-expense-not-found = Impossibile trovare la spesa #{$number} per mostrare i dettagli.

## /transfer

transfer-ok = Trasferimento registrato con successo.
transfer-receiver-not-found = Impossibile trovare il viaggiatore "{$name}" a cui trasferire denaro.
transfer-sender-not-found = Impossibile trovare il viaggiatore "{$name}" da cui trasferire denaro.

## /deletetransfer

delete-transfer-not-found = Impossibile trovare il trasferimento #{$number} da eliminare.
delete-transfer-ok = Trasferimento #{$number} eliminata con successo.

## /listtransfers

list-transfers-name-not-found = Nessun trasferimento relativo al viaggiatore "{$name}" trovato.
list-transfers-not-found = Nessun trasferimento trovato. Usa `/{-transfer-command} <mittente> <destinatario> <importo>` per aggiungerne uno.

## unknown command

invalid-command-usage = 
    Uso non valido del comando: {$command}.

    {$help-message}
unknown-command = 
    Comando sconosciuto: {$command}.
unknown-command-best-match = 
    Comando sconosciuto: {$command}.
    Intendevi: {$best-match}?
