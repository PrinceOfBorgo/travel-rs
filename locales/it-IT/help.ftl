## Comando

descr-command = Questi comandi sono supportati:

## /help

descr-help = Mostra un messaggio di aiuto per il comando specificato. Se non viene specificato alcun comando, mostra le descrizioni di tutti i comandi.
help-help = 
    /{-help-command} — {descr-help}

    Uso: /{-help-command} [comando]

## /setlanguage

descr-set-language = Imposta la lingua predefinita del bot per questa chat.
help-set-language =
    /{-set-language-command} — {descr-set-language}

    Lingue disponibili:
    {$available-langs}

    Uso: /{-set-language-command} [lingua]

## /setcurrency

descr-set-currency = Imposta la valuta predefinita per questa chat.
help-set-currency =
    /{-set-currency-command} — {descr-set-currency}

    Uso: /{-set-currency-command} [codice valuta]

## /addtraveler

descr-add-traveler = Aggiungi un viaggiatore con il nome specificato al piano di viaggio.
help-add-traveler =
    /{-add-traveler-command} — {descr-add-traveler}

    Uso: /{-add-traveler-command} <nome>

## /deletetraveler

descr-delete-traveler = Elimina il viaggiatore con il nome specificato dal piano di viaggio.
help-delete-traveler =
    /{-delete-traveler-command} — {descr-delete-traveler}

    Uso: /{-delete-traveler-command} <nome>

## /listtravelers

descr-list-travelers = Mostra i viaggiatori nel piano di viaggio.
help-list-travelers = 
    /{-list-travelers-command} — {descr-list-travelers}

    Uso: /{-list-travelers-command}

## /addexpense

descr-add-expense = Avvia una nuova sessione interattiva per aggiungere una spesa al piano di viaggio.
help-add-expense = 
    /{-add-expense-command} — {descr-add-expense}

    Durante la sessione, ti verrà chiesto di:
    - Inviare un messaggio con la descrizione della spesa.
    - Inviare un messaggio con l'importo della spesa.
    - Inviare un messaggio con il nome del viaggiatore che ha pagato la spesa.
    - Inviare un messaggio con i viaggiatori che partecipano alla spesa e le loro quote.

    Il processo può essere interrotto in qualsiasi momento inviando `/{-cancel-command}`. 

    Per dividere la spesa tra più viaggiatori puoi:
    - Inviare un messaggio per ciascun viaggiatore con cui vuoi condividere la spesa, o specificare più viaggiatori separandoli con `{-split-among-entries-sep}`.
    - Utilizzare il formato `<nome>{-split-among-name-amount-sep} <importo>` dove `<importo>` può essere seguito da `%` se è una percentuale dell'importo residuo.
    > Esempio: `Alice{-split-among-name-amount-sep} 50`, `Bob{-split-among-name-amount-sep} 20%`, `Charles`, `John{-split-among-name-amount-sep} 30{-split-among-entries-sep} Jane{-split-among-name-amount-sep} 10%` sono tutte sintassi valide.
    > Esempio: Se il totale è `100`, digitando `Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-name-amount-sep} 60%` significa che Alice pagherà `40` quindi il residuo è `60`, Bob pagherà `24` (cioè il 40% di 60) e Charles pagherà `36` (cioè il 60% di 60).

    - Omettere `{-split-among-name-amount-sep} <importo>` se vuoi dividere equamente la spesa residua tra i viaggiatori.
    > Esempio: L'input `Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-entries-sep} David` è equivalente a impostare sia Charles che David con importi del 30%.

    - Inserire `{-all-kword}` per dividerlo equamente tra tutti i viaggiatori.

    Uso: /{-add-expense-command}

## /deleteexpense

descr-delete-expense = Elimina la spesa con il numero identificativo specificato dal piano di viaggio.
help-delete-expense = 
    /{-delete-expense-command} — {descr-delete-expense}

    Uso: /{-delete-expense-command} <numero>

## /listexpenses

descr-list-expenses = Mostra le spese nel piano di viaggio.
help-list-expenses = 
    /{-list-expenses-command} — {descr-list-expenses}

    Uso: /{-list-expenses-command}

## /findexpenses

descr-find-expenses = Cerca le spese che corrispondono alla descrizione fornita. Supporta la ricerca fuzzy per un abbinamento più flessibile.
help-find-expenses = 
    /{-find-expenses-command} — {descr-find-expenses}

    Uso: /{-find-expenses-command} <descrizione>

## /showexpense

descr-show-expense = Mostra i dettagli della spesa con il numero identificativo specificato.
help-show-expense = 
    /{-show-expense-command} — {descr-show-expense}

    Uso: /{-show-expense-command} <numero>

## /transfer

descr-transfer = Trasferisci l'importo specificato da un viaggiatore a un altro.
help-transfer = 
    /{-transfer-command} — {descr-transfer}

    Uso: /{-transfer-command} <mittente> <destinatario> <importo>

## /showbalance

descr-show-balance = Mostra il saldo semplificato del viaggiatore specificato, minimizzando il numero totale di trasferimenti necessari a saldare i debiti.
help-show-balance = 
    /{-show-balance-command} — {descr-show-balance}

    Uso: /{-show-balance-command} <nome>

## /showbalances

descr-show-balances = Mostra i saldi semplificati di tutti i viaggiatori, minimizzando il numero totale di trasferimenti necessari a saldare i debiti.
help-show-balances = 
    /{-show-balances-command} — {descr-show-balances}

    Uso: /{-show-balances-command}

## /cancel

descr-cancel = Annulla il comando interattivo attualmente in esecuzione.
help-cancel = 
    /{-cancel-command} — {descr-cancel}

    Uso: /{-cancel-command}
