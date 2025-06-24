format-share-details = - {$traveler-name}: {$amount}
format-expense-details =
    Numero: {$number} - Descrizione: {$description}
    Registrata il: {DATETIME($datetime, dateStyle: "long")}
    Importo: {$amount}
    Pagato da: {$creditor}
    Quote:
    {$shares}
format-expense = [{DATETIME($datetime, dateStyle: "short")}] Spesa #{$number}: {$description} - {$amount}
format-transfer = [{DATETIME($datetime, dateStyle: "short")}] Trasferimento #{$number}: {$sender} ha inviato {$amount} a {$receiver}
format-traveler-stats-amount = {$traveler-name}: {$amount}
format-traveler-stats-frequency =
    {$traveler-name}: {$count ->
        [one] {$count} spesa
       *[other] {$count} spese
    }
format-average-per-day = {$amount} al giorno [da {DATETIME($oldest-timestamp, dateStyle: "short")} ad oggi {DATETIME($now, dateStyle: "short")}]
format-expense-stats =
    Statistiche sulle spese:
    { $count ->
        [0] - Numero: {$count}
        *[other]
            - Numero: {$count}
            - Somma: {$sum}
            - Media: {$mean}
            - Spese massime: {$max}
            - Spese minime: {$min}
            - Media al giorno: {$average-per-day}
            - Più vecchia: {$oldest}
            - Più recente: {$newest}
    }
format-transfer-stats =
    Statistiche sui trasferimenti:
    { $count ->
        [0] - Numero: {$count}
        *[other]
            - Numero: {$count}
            - Somma: {$sum}
            - Media: {$mean}
            - Trasferimenti massimi: {$max}
            - Trasferimenti minimi: {$min}
            - Media al giorno: {$average-per-day}
            - Più vecchio: {$oldest}
            - Più recente: {$newest}
    }
format-traveler-stats =
    Statistiche sui viaggiatori:
    { $count ->
        [0] - Numero: {$count}
        *[other] 
            - Numero: {$count}{ $expenses-count ->
                [0] {""}
                *[other]
                    {" "}
                    - Chi ha pagato di più: {$travelers-paid-most}
                    - Chi ha pagato di meno: {$travelers-paid-least}
                    - Chi paga più frequentemente: {$travelers-pays-most-frequently}
                    - Chi paga meno frequentemente: {$travelers-pays-least-frequently}
            }{ $balances-count ->
                [0] {""}
                *[other]
                    {" "}
                    - Maggiori creditori: {$major-creditors}
                    - Maggiori debitori: {$major-debtors}
            }
    }
format-stats =
    {$expense-stats}

    {$transfer-stats}

    {$traveler-stats}
