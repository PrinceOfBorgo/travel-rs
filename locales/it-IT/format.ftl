format-share-details = • <b>{$traveler-name}</b>: {$amount}
format-expense-details =
    <b>Numero:</b> {$number} - <b>Descrizione:</b> <i>{$description}</i>
    <b>Registrata il:</b> {DATETIME($datetime, dateStyle: "long")}
    <b>Importo:</b> {$amount}
    <b>Pagato da:</b> {$creditor}
    <b>Quote:</b>
    {$shares}
format-expense = [{DATETIME($datetime, dateStyle: "short")}] <b>Spesa #{$number}</b>: <i>{$description}</i> - <b>{$amount}</b>
format-transfer = [{DATETIME($datetime, dateStyle: "short")}] <b>Trasferimento #{$number}</b>: <b>{$sender}</b> ha inviato <b>{$amount}</b> a <b>{$receiver}</b>
format-traveler-stats-amount = ◦ <b>{$traveler-name}</b>: {$amount}
format-traveler-stats-frequency =
    ◦ <b>{$traveler-name}</b>: {$count ->
        [one] {$count} spesa
       *[other] {$count} spese
    }
format-average-per-day = {$amount} al giorno [da {DATETIME($oldest-timestamp, dateStyle: "short")} ad oggi {DATETIME($now, dateStyle: "short")}]
format-expense-stats =
    <b>Statistiche sulle spese:</b>
    { $count ->
        [0] • <b>Numero:</b> {$count}
        *[other]
            • <b>Numero:</b> {$count}
            • <b>Somma:</b> {$sum}
            • <b>Media:</b> {$mean}
            • <b>Spese massime:</b> {$max}
            • <b>Spese minime:</b> {$min}
            • <b>Media al giorno:</b> {$average-per-day}
            • <b>Più vecchia:</b> {$oldest}
            • <b>Più recente:</b> {$newest}
    }
format-transfer-stats =
    <b>Statistiche sui trasferimenti:</b>
    { $count ->
        [0] • <b>Numero:</b> {$count}
        *[other]
            • <b>Numero:</b> {$count}
            • <b>Somma:</b> {$sum}
            • <b>Media:</b> {$mean}
            • <b>Trasferimenti massimi:</b> {$max}
            • <b>Trasferimenti minimi:</b> {$min}
            • <b>Media al giorno:</b> {$average-per-day}
            • <b>Più vecchio:</b> {$oldest}
            • <b>Più recente:</b> {$newest}
    }
format-traveler-stats =
    <b>Statistiche sui viaggiatori:</b>
    { $count ->
        [0] • <b>Numero:</b> {$count}
        *[other] 
            • <b>Numero:</b> {$count}{ $expenses-count ->
                [0] {""}
                *[other]
                    {" "}
                    • <b>Chi ha pagato di più:</b> {$travelers-paid-most}
                    • <b>Chi ha pagato di meno:</b> {$travelers-paid-least}
                    • <b>Chi paga più frequentemente:</b> {$travelers-pays-most-frequently}
                    • <b>Chi paga meno frequentemente:</b> {$travelers-pays-least-frequently}
            }{ $balances-count ->
                [0] {""}
                *[other]
                    {" "}
                    • <b>Maggiori creditori:</b> {$major-creditors}
                    • <b>Maggiori debitori:</b> {$major-debtors}
            }
    }
format-stats =
    {$expense-stats}

    {$transfer-stats}

    {$traveler-stats}
