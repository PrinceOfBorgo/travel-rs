-- Add timestamp fields to the expense and transferred_to tables
DEFINE FIELD timestamp_utc ON expense VALUE time::now() PERMISSIONS FOR select FULL, FOR create, update NONE;
DEFINE FIELD timestamp_utc ON transferred_to VALUE time::now() PERMISSIONS FOR select FULL, FOR create, update NONE;

-- Update existing records to set the timestamp_utc field to the current time
-- This is a one-time operation to ensure all existing records have a timestamp
UPDATE expense SET timestamp_utc = time::now() WHERE timestamp_utc IS NULL;
UPDATE transferred_to SET timestamp_utc = time::now() WHERE timestamp_utc IS NULL;

-- Redefine functions to retrieve expenses and transfers with timestamps
DEFINE FUNCTION OVERWRITE fn::get_expense_details($chat: record<chat>, $expense_number: int) { RETURN (SELECT amount AS expense_amount, number AS expense_number, description AS expense_description, chat AS chat, timestamp_utc as timestamp_utc, (<-paid_for<-traveler)[0].name AS creditor_name, array::group((SELECT in.name AS traveler_name, amount FROM split WHERE out = $parent.id ORDER BY amount DESC, traveler_name ASC)) AS shares FROM expense WHERE chat = $chat AND number = $expense_number); } COMMENT 'Retrieve the details of the expense with the given identifying number in the specified chat' PERMISSIONS FULL;
DEFINE FUNCTION OVERWRITE fn::get_transfers($chat: record<chat>) { RETURN SELECT number, amount, in.name AS sender_name, out.name AS receiver_name, in.chat AS chat, timestamp_utc FROM transferred_to WHERE in.chat.id = $chat; } COMMENT 'Retrieve the transfers recorded in the specified chat' PERMISSIONS FULL;
