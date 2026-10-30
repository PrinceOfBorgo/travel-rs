## /add_traveler

add-traveler-ask-name = Qual è il nome del viaggiatore? Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.
add-traveler-invalid-name = Hai inviato un nome non valido, per favore riprova.

## /delete_traveler

delete-traveler-ask-name = Quale viaggiatore vuoi eliminare? Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.
delete-traveler-invalid-name = Hai inviato un nome non valido, per favore riprova.
delete-traveler-confirm =
    Sei sicuro di voler eliminare il viaggiatore <b>{$name}</b>?
    ⚠️ <i>Tutti i trasferimenti che coinvolgono questo viaggiatore verranno eliminati.</i>

## /delete_expense

delete-expense-ask-number = Quale spesa vuoi eliminare? Invia il numero della spesa. Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.
delete-expense-invalid-number = Hai inviato un numero non valido, per favore riprova.
delete-expense-confirm = Sei sicuro di voler eliminare la spesa <b>#{$number}</b>?

## /show_expense

show-expense-ask-number = Quale spesa vuoi visualizzare? Invia il numero della spesa. Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.
show-expense-invalid-number = Hai inviato un numero non valido, per favore riprova.

## /delete_transfer

delete-transfer-ask-number = Quale trasferimento vuoi eliminare? Invia il numero del trasferimento. Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.
delete-transfer-invalid-number = Hai inviato un numero non valido, per favore riprova.
delete-transfer-confirm = Sei sicuro di voler eliminare il trasferimento <b>#{$number}</b>?

## /set_language

set-language-ask-langid = Quale lingua vuoi impostare?
set-language-invalid-langid = Hai inviato un identificatore di lingua non valido, per favore riprova.

## /set_currency

set-currency-ask-currency = Quale valuta vuoi impostare? Selezionane una dalla lista sottostante o invia il codice valuta se non presente.
set-currency-invalid-currency = Hai inviato un codice valuta non valido, per favore riprova.

## /list_expenses

list-expenses-ask-description = Invia una descrizione (o parte di essa) per filtrare le spese.

## /transfer

transfer-ask-from = Chi è il mittente? Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.
transfer-ask-from-reprompt = Hai inviato un nome non valido, per favore riprova. Chi è il mittente?
transfer-from-not-found = Viaggiatore <b>{$name}</b> non trovato. Chi è il mittente?
transfer-ask-to = Chi è il destinatario del trasferimento da <b>{$name}</b>?
transfer-ask-to-reprompt = Hai inviato un nome non valido, per favore riprova. Chi è il destinatario?
transfer-to-not-found = Viaggiatore <b>{$name}</b> non trovato. Chi è il destinatario?
transfer-ask-amount = Quanto ha trasferito <b>{$name}</b>?
transfer-invalid-amount = Hai inviato un importo non valido, per favore riprova.

## /cleartravelers

clear-travelers-confirm =
    Sei sicuro di voler eliminare <b>tutti i viaggiatori</b>?
    ⚠️ <i>Tutti i trasferimenti che coinvolgono questi viaggiatori verranno eliminati.</i>
clear-travelers-show-expenses-prompt = Seleziona un viaggiatore per visualizzare le sue spese, oppure tocca <b>Tutti</b> per visualizzare tutte.

## /clearexpenses

clear-expenses-confirm = Sei sicuro di voler eliminare <b>tutte le spese</b>?

## /cleartransfers

clear-transfers-confirm = Sei sicuro di voler eliminare <b>tutti i trasferimenti</b>?

## /clearall

clear-all-confirm =
    Sei sicuro di voler eliminare <b>tutti i viaggiatori, spese e trasferimenti</b>?
    ⚠️ <i>Questa azione non può essere annullata.</i>
