## Comando

descr-command = <b>Questi comandi sono supportati:</b>

## /help

descr-help = Mostra un messaggio di aiuto per il comando specificato. Se non viene specificato alcun comando, mostra le descrizioni di tutti i comandi.
help-help = 
    <b>/{-help-command}</b> — {descr-help}

    <b>Uso:</b> <code>/{-help-command} [comando]</code>

## /setlanguage

descr-set-language = Imposta la lingua predefinita del bot per questa chat.
help-set-language =
    <b>/{-set-language-command}</b> — {descr-set-language}

    <b>Lingue disponibili:</b>
    {$available-langs}

    <b>Uso:</b> <code>/{-set-language-command} &lt;lingua&gt;</code>

## /setcurrency

descr-set-currency = Imposta la valuta predefinita per il piano di viaggio.
help-set-currency =
    <b>/{-set-currency-command}</b> — {descr-set-currency}

    <b>Uso:</b> <code>/{-set-currency-command} &lt;codice valuta&gt;</code>

## /addtraveler

descr-add-traveler = Aggiungi un viaggiatore con il nome specificato al piano di viaggio.
help-add-traveler =
    <b>/{-add-traveler-command}</b> — {descr-add-traveler}

    <b>Uso:</b> <code>/{-add-traveler-command} &lt;nome&gt;</code>

## /deletetraveler

descr-delete-traveler = Elimina il viaggiatore con il nome specificato (e i trasferimenti associati) dal piano di viaggio.
help-delete-traveler =
    <b>/{-delete-traveler-command}</b> — {descr-delete-traveler}

    <b>Uso:</b> <code>/{-delete-traveler-command} &lt;nome&gt;</code>

## /listtravelers

descr-list-travelers = Mostra i viaggiatori nel piano di viaggio.
help-list-travelers = 
    <b>/{-list-travelers-command}</b> — {descr-list-travelers}

    <b>Uso:</b> <code>/{-list-travelers-command}</code>

## /addexpense

descr-add-expense = Avvia una nuova sessione interattiva per aggiungere una spesa al piano di viaggio.
help-add-expense = 
    <b>/{-add-expense-command}</b> — {descr-add-expense}

    Durante la sessione, ti verrà chiesto di:
    1. Inviare un messaggio con la descrizione della spesa.
    2. Inviare un messaggio con l'importo della spesa.
    3. Inviare un messaggio con il nome del viaggiatore che ha pagato la spesa.
    4. Inviare uno o più messaggi con i viaggiatori che partecipano alla spesa e le loro quote.

    Il processo può essere interrotto in qualsiasi momento inviando <code>/{-cancel-command}</code>.

    Per dividere la spesa tra più viaggiatori puoi:
    • Inviare un messaggio per ciascun viaggiatore con cui vuoi condividere la spesa, o specificare più viaggiatori separandoli con <code>{-split-among-entries-sep}</code>.
    • Utilizzare il formato <code>&lt;nome&gt;{-split-among-name-amount-sep} &lt;importo&gt;</code> dove <code>&lt;importo&gt;</code> può essere seguito da <code>%</code> se è una percentuale dell'importo residuo.
    <blockquote><i>Esempio:</i> <code>Alice{-split-among-name-amount-sep} 50</code>, <code>Bob{-split-among-name-amount-sep} 20%</code>, <code>Charles</code>, <code>John{-split-among-name-amount-sep} 30{-split-among-entries-sep} Jane{-split-among-name-amount-sep} 10%</code> sono tutte sintassi valide.
    <i>Esempio:</i> Se il totale è <code>100</code>, digitando <code>Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-name-amount-sep} 60%</code> significa che Alice pagherà <code>40</code> quindi il residuo è <code>60</code>, Bob pagherà <code>24</code> (cioè il 40% di 60) e Charles pagherà <code>36</code> (cioè il 60% di 60).</blockquote>

    • Omettere <code>{-split-among-name-amount-sep} &lt;importo&gt;</code> se vuoi dividere equamente la spesa residua tra i viaggiatori.
    <blockquote><i>Esempio:</i> L'input <code>Alice{-split-among-name-amount-sep} 40{-split-among-entries-sep} Bob{-split-among-name-amount-sep} 40%{-split-among-entries-sep} Charles{-split-among-entries-sep} David</code> è equivalente a impostare sia Charles che David con importi del 30%.</blockquote>

    • Inserire <code>{-all-kword}</code> per dividerlo equamente tra tutti i viaggiatori.

    <b>Uso:</b> <code>/{-add-expense-command}</code>

## /deleteexpense

descr-delete-expense = Elimina la spesa con il numero identificativo specificato dal piano di viaggio.
help-delete-expense = 
    <b>/{-delete-expense-command}</b> — {descr-delete-expense}

    <b>Uso:</b> <code>/{-delete-expense-command} &lt;numero&gt;</code>

## /listexpenses

descr-list-expenses = Mostra le spese nel piano di viaggio. Se viene specificata una descrizione, mostra solo le spese che corrispondono alla descrizione fornita. Supporta la ricerca approssimativa (fuzzy search) per una corrispondenza più flessibile.
help-list-expenses = 
    <b>/{-list-expenses-command}</b> — {descr-list-expenses}

    <b>Uso:</b> <code>/{-list-expenses-command} [descrizione]</code>

## /showexpense

descr-show-expense = Mostra i dettagli della spesa con il numero identificativo specificato.
help-show-expense = 
    <b>/{-show-expense-command}</b> — {descr-show-expense}

    <b>Uso:</b> <code>/{-show-expense-command} &lt;numero&gt;</code>

## /transfer

descr-transfer = Trasferisci l'importo specificato da un viaggiatore a un altro.
help-transfer = 
    <b>/{-transfer-command}</b> — {descr-transfer}

    <b>Uso:</b> <code>/{-transfer-command} &lt;mittente&gt; &lt;destinatario&gt; &lt;importo&gt;</code>

## /deletetransfer

descr-delete-transfer = Elimina il trasferimento con il numero identificativo specificato dal piano di viaggio.
help-delete-transfer = 
    <b>/{-delete-transfer-command}</b> — {descr-delete-transfer}

    <b>Uso:</b> <code>/{-delete-transfer-command} &lt;numero&gt;</code>

## /listtransfers

descr-list-transfers = Mostra i trasferimenti nel piano di viaggio. Se viene specificato un nome, mostra solo i trasferimenti che coinvolgono quel viaggiatore.
help-list-transfers = 
    <b>/{-list-transfers-command}</b> — {descr-list-transfers}

    <b>Uso:</b> <code>/{-list-transfers-command} [nome]</code>

## /showbalances

descr-show-balances = Mostra i saldi semplificati di tutti i viaggiatori, minimizzando il numero totale di trasferimenti necessari per saldare i debiti. Se viene specificato un nome, mostra il saldo del viaggiatore specificato.
help-show-balances = 
    <b>/{-show-balances-command}</b> — {descr-show-balances}

    <b>Uso:</b> <code>/{-show-balances-command} [nome]</code>

## /showstats
descr-show-stats = Mostra le statistiche per spese, trasferimenti e viaggiatori nel piano di viaggio.
help-show-stats = 
    <b>/{-show-stats-command}</b> — {descr-show-stats}

    <b>Uso:</b> <code>/{-show-stats-command}</code>

## /cleartravelers

descr-clear-travelers = Elimina tutti i viaggiatori e i trasferimenti associati dal piano di viaggio.
help-clear-travelers =
    <b>/{-clear-travelers-command}</b> — {descr-clear-travelers}

    <b>Uso:</b> <code>/{-clear-travelers-command}</code>

## /clearexpenses

descr-clear-expenses = Elimina tutte le spese dal piano di viaggio.
help-clear-expenses =
    <b>/{-clear-expenses-command}</b> — {descr-clear-expenses}

    <b>Uso:</b> <code>/{-clear-expenses-command}</code>

## /cleartransfers

descr-clear-transfers = Elimina tutti i trasferimenti dal piano di viaggio.
help-clear-transfers =
    <b>/{-clear-transfers-command}</b> — {descr-clear-transfers}

    <b>Uso:</b> <code>/{-clear-transfers-command}</code>

## /clearall

descr-clear-all = Elimina tutti i viaggiatori, spese e trasferimenti dal piano di viaggio.
help-clear-all =
    <b>/{-clear-all-command}</b> — {descr-clear-all}

    <b>Uso:</b> <code>/{-clear-all-command}</code>

## /cancel

descr-cancel = Annulla il comando interattivo attualmente in esecuzione.
help-cancel = 
    <b>/{-cancel-command}</b> — {descr-cancel}

    <b>Uso:</b> <code>/{-cancel-command}</code>
