
## Roadmap

- [ ] Currency support: exchange using external APIs.
- [ ] Add localization
  - [ ] Add languages.
  - [ ] Add currency formatting (decimal separators).
- [ ] Logs with span_id/trace_id.
- [ ] Formatting of bot responses (bold, italic, etc.).
- [ ] Improve error handling: distinguish errors resulting from DB queries.
- [ ] Delete from DB always returns OK even if the entry doesn't exist. Should return WARN with a dedicated message.
- [ ] Refactor code to make it more readable and maintainable.
- [ ] Add tests (not sure what to test).