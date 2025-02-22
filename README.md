
## Roadmap

- [x] Use stable channel when `std::sync::LazyLock` will be stabilized (1.80 - 25 July, 2024).
- [ ] Currency support: exchange using external APIs.
- [ ] Add localization
  - [ ] Add languages.
  - [ ] Add currency formatting (decimal separators).
- [ ] Logs with span_id/trace_id.
- [ ] Formatting of bot responses (bold, italic, etc.). https://docs.rs/teloxide/latest/teloxide/types/enum.ParseMode.html
- [ ] Improve error handling: distinguish errors resulting from DB queries.
  - [x] Delete from DB always returns OK even if the entry doesn't exist. Should return WARN with a dedicated message.
- [ ] Refactor code to make it more readable and maintainable.
- [ ] Add tests (not sure what to test).
- [ ] Add assertion/events to check if `in.chat = out.chat` for `split` and `paid_for` tables.
- [x] Handle `unknown_command` when command name is right but arguments are incomplete.
- [x] Add dedicated help messages for all commands
  - [x] Help
  - [x] AddTraveler
  - [x] DeleteTraveler
  - [x] ListTravelers
  - [x] AddExpense
  - [x] DeleteExpense
  - [x] ListExpenses
  - [x] FindExpenses
  - [x] Transfer
  - [x] Cancel
