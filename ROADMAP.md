# Roadmap

## To do

- [ ] Currency support:
  - [ ] Exchange using external APIs. https://crates.io/crates/freecurrencyapi-rs
  - [x] Handle different currencies with their specific precision instead of Decimal.
- [ ] Add localization:
  - [ ] Add languages:
    - [x] en-US
    - [x] it-IT
  - [ ] Add currency input formatting (decimal separators).
  - [x] Add `SetLanguage` command.
- [ ] Formatting of bot responses (bold, italic, etc.). https://docs.rs/teloxide/latest/teloxide/types/enum.ParseMode.html
- [ ] Improve error handling: distinguish errors resulting from DB queries.
  - [x] Delete from DB always returns OK even if the entry doesn't exist. Should return WARN with a dedicated message.
- [ ] Refactor code to make it more readable and maintainable (constantly on-going).
- [ ] Add assertion/events to check if `in.chat = out.chat` for relationships.
- [ ] Improve user experience:
  - [ ] Implement a Telegram Mini App.
  - [x] Add inline keyboard support.
  - [x] Interactive prompts for commands invoked without their arguments.
- [ ] Add `EditExpense` command.
- [ ] Change `ListExpenses` command so that the filter matches the string representation of the expense.
- [ ] Refactor callback prefix to derive them from commands instead of hardcoding them.
- [x] Add "clear" commands:
  - [x] `ClearTravelers`
  - [x] `ClearExpenses`
  - [x] `ClearTransfers`
  - [x] `ClearAll` (clears travelers, expenses, and transfers)
- [ ] Handle multiple travel plans in one chat.

## Done

- [x] Use stable channel when `std::sync::LazyLock` will be stabilized (1.80 - 25 July, 2024).
- [x] Add tests:
  - [x] Unit tests:
    - [x] DB connection.
    - [x] Command parsing.
    - [x] Traveler name parsing.
    - [x] Commands.
  - [x] Integration tests.
- [x] Handle `unknown_command` when command name is right but arguments are incomplete.
- [x] Add dedicated help messages for all commands.
- [x] Add timestamps to expenses and transfers.
- [x] Add `ShowStats` command.
- [x] Factor callback dispatch branches out of `main.rs` into their dialogue modules.
- [x] Logs with trace_id.
- [x] Add confirmation step to delete commands before executing the deletion.
