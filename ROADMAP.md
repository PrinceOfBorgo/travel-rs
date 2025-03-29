
## Roadmap

- [x] Use stable channel when `std::sync::LazyLock` will be stabilized (1.80 - 25 July, 2024).
- [ ] Currency support
  - [ ] Exchange using external APIs. https://crates.io/crates/freecurrencyapi-rs
  - [x] Handle different currencies with their specific precision instead of Decimal.
- [ ] Add localization
  - [ ] Add languages:
    - [x] en-US
    - [x] it-IT
  - [ ] Add currency input formatting (decimal separators).
  - [x] Add `SetLanguage` command.
- [ ] Logs with span_id/trace_id.
- [ ] Formatting of bot responses (bold, italic, etc.). https://docs.rs/teloxide/latest/teloxide/types/enum.ParseMode.html
- [ ] Improve error handling: distinguish errors resulting from DB queries.
  - [x] Delete from DB always returns OK even if the entry doesn't exist. Should return WARN with a dedicated message.
- [ ] Refactor code to make it more readable and maintainable.
- [ ] Add tests (not sure what to test).
- [ ] Add assertion/events to check if `in.chat = out.chat` for relationships.
- [x] Handle `unknown_command` when command name is right but arguments are incomplete.
- [x] Add dedicated help messages for all commands.
- [ ] Improve user experience (e.g. add buttons).
- [ ] Find a way to use constants instead of string literals in `#[command(description = "here")]` syntax.
- [ ] Handle deletions of travelers from the database:
  - Prevent deleting if related to expenses?
  - Delete expense when `paid_for` is deleted?
  - Rework database? Remove `paid_for` relation in favor of new field `paid_by` in `expense` table.
  - How to handle removed travelers that are related to expenses through `split`? Sum of shares will not equal the expense amount anymore.
