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
