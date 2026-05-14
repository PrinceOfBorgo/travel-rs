# Changelog

## [0.3.1-SNAPSHOT] - Unreleased
### Added
- Every instrumented handler span now includes a `trace_id` field (monotonically increasing process-unique counter) for correlating log lines across nested calls.
- More INFO and WARN-level summary logs for production visibility.
- [`DEPLOYMENT.md`](DEPLOYMENT.md) guide covering fresh installation, upgrade procedures, rollback, and a migration reference table. Includes both Docker and manual (direct binary / systemd) deployment paths.
- The `release.yml` workflow now builds and attaches a `deploy-v<version>.zip` bundle to each GitHub Release, containing a version-pinned `docker-compose.yml`, up-to-date locale files, database schema and migration scripts, a sanitized profile template, and a `MIGRATIONS.md` listing the migrations required for that release.
- Confirmation step for destructive commands: `/deleteexpense`, `/deletetraveler`, and `/deletetransfer` now show a Yes/No inline keyboard asking the user to confirm before executing the deletion. This applies to both the interactive dialogue and the inline form (e.g. `/deleteexpense 5`). Pressing "No" cancels the operation.
- Clear commands: `/clearexpenses`, `/cleartransfers`, `/cleartravelers`, and `/clearall` to bulk-delete all expenses, transfers, travelers, or everything at once. Each shows a confirmation prompt with a Yes/No keyboard before executing. `/cleartravelers` and `/clearall` warn that associated data will also be deleted.
- `/cleartravelers`: when travelers with expenses exist, a traveler-picker inline keyboard is shown to view the blocking expenses per traveler (or all at once); if only one traveler has expenses, their expenses are listed directly without the keyboard.
- Travelers now have a stable, auto-incremented `number` field (analogous to expenses and transfers). Inline-keyboard callbacks use this numeric ID instead of the traveler name to keep the callback messages short. This requires [database](database) schema updates. Run the following scripts to migrate:
  - [`008_add_traveler_number.surql`](database/migrations/008_add_traveler_number.surql)

### Changed
- Updated `README.md` structure: added Deployment section linking to `DEPLOYMENT.md` and renumbered subsequent sections.

### Fixed
- The bot now pushes the command list to the global (default) scope at startup, so the command menu is available immediately in new chats without waiting for the first message.
- Traveler-picker inline keyboards no longer suffer from callback messages exceeding Telegram's 64-byte limit: callbacks now carry a compact progressive numeric ID instead of the traveler name, ensuring messages are always short enough.

## [0.3.0] - 2026-05-10
### ✨ Minor Release
### Added
- Command descriptions shown by Telegram are now translated into the chat's selected language, and update automatically when the language is changed via `/setlanguage`.
- `dialogues/pending_commands.ftl` and `labels.ftl` localization files.
- `CommandArg` and `CommandOutcome` enums for structured command handling.
- `/setlanguage`, `/setcurrency`, `/addtraveler`, `/deletetraveler`, `/deleteexpense`, `/showexpense`, `/deletetransfer` interactive dialogues for cases when commands are invoked without arguments.
- `/transfer` interactive dialogue: when invoked without arguments (or with partial arguments), guides the user through selecting a sender, receiver, and amount via a multi-step conversation with traveler-picker inline keyboards. Partial invocations (`/transfer Alice`, `/transfer Alice Bob`) skip already-provided steps. Invalid or non-existent traveler names are re-prompted with the keyboard.
- `/listexpenses` now shows a "Filter…" inline keyboard button when invoked without a description. Tapping it starts an interactive dialogue that asks for search criteria (or `/cancel` to abort).
- `/help`, `/listtransfers`, and `/showbalances` display inline keyboards for quick selection when invoked without arguments.
- `/deleteexpense`, `/showexpense`, and `/deletetransfer` interactive dialogues now show a paginated inline keyboard listing the chat's expenses or transfers for quick selection (with `◀`/`▶` page navigation); free-text input is still accepted as a fallback.
- `/addexpense` split step now includes a Help button (❓) that displays the full `/addexpense` usage guide without interrupting the dialogue.
- Inline keyboard selections now echo the chosen value on the original prompt message (e.g. `✓ Alice`) so users can see what they picked after the keyboard is dismissed.
- Input validations:
  - `/setcurrency` rejects unknown codes (must be a known ISO 4217 or crypto code).
  - `/transfer` rejects non-positive amounts and self-transfers (sender == receiver).
  - `/addexpense` dialogue rejects non-positive amounts and trims/rejects empty descriptions.
  - Traveler names: rejected if empty (after trim) or if they contain control/invisible characters (in addition to the existing slash/reserved-keyword/invalid-character checks).
  - Traveler names are now compared case-insensitively within a chat, so `Alice` and `alice` are treated as the same name (preventing duplicates that differ only in letter case).
- Defense-in-depth schema constraints mirroring the application-level validations. This functionality requires [database](database) schema updates. Run the following scripts to migrate:
  - [`006_add_validation_constraints.surql`](database/migrations/006_add_validation_constraints.surql)
  - [`007_case_insensitive_traveler_names.surql`](database/migrations/007_case_insensitive_traveler_names.surql)

### Changed
- Added disk space cleanup step to `ci.yml` workflow.
- Fixed descriptions for `dev-local` and `dev-local-docker` profiles settings files.
- Replaced the batch script `start_docker_db.bat` with an enhanced PowerShell script. The new `start_docker_db.ps1` contains configurable variables for Docker parameters and retrieves the SurrealDB version from `Cargo.toml` to ensure a compatible Docker image is used.
- Reworked `README.md` for clarity and updated structure.
- `/setlanguage`, `/setcurrency`, `/addtraveler`, `/deletetraveler`, `/deleteexpense`, `/showexpense`, `/deletetransfer` now prompt for their arguments when invoked without one.
- `/transfer` now prompts for sender, receiver, and amount when invoked without (or with partial) arguments. Each step shows a traveler-picker inline keyboard; free-text input is accepted as a fallback. Non-existent travelers are caught early and re-prompted instead of failing at the final step.
- `/setlanguage` and `/setcurrency` interactive dialogues now offer an inline keyboard for quick selection (configurable short list for `/setcurrency`); free-text input is still accepted as a fallback.
- `/deletetraveler` interactive dialogue now shows an inline keyboard with the chat's travelers for quick selection; free-text input is still accepted as a fallback.
- Heavily refactored dialogues and storages handling; `/cancel` works for any dialogue.
- The "another process is already running" notice and the `/cancel` confirmation now identify the dialogue in question (e.g. `Another process (/addexpense) is already running, ...`, `The process (/addtraveler) was cancelled.`). Each dialogue state implements a shared `DialogueState` trait that exposes its user-facing label.
- `/listtransfers` and `/showbalances` now accept their `name` argument as optional via `CommandArg<Name>`.
- Removed `utils.rs`: tracing attribute aliases moved to `macros.rs`, debt logic (`update_debts`, `simplify_balances`) moved to `debt.rs`, and `indent_multiline` moved to `i18n/mod.rs`.
- Refactored inline-keyboard utilities from a single `keyboard.rs` file into a dedicated `keyboard` module with submodules (`callback`, `travelers`, `paginated`) for better maintainability.
- Factored inline callback-dispatch branches out of `main.rs` into `pending_command_dialogue::callback_branch()` and `add_expense_dialogue::callback_branch()`, each with a corresponding `is_*_callback()` filter function.
- Removed the `CommandError::EmptyInput` variant and its associated localization key (`command-error-empty-input`). Commands that previously returned this error now start an interactive dialogue instead.

### Fixed
- Pending-command dialogues now re-prompt the user when the underlying command fails, instead of silently leaving the dialogue alive with no indication that retrying is possible.

## [0.2.5] - 2026-04-03
### 🔧 Patch Release
### Added
- Optional chat IDs whitelist under `[bot]` section in (profile) settings files. If specified, only the whitelisted chats will be allowed to interact with the bot.
- Debug log on startup to show the loaded settings.
- Added `config/profiles/dev-cloud.toml` template and extended local profiles documentation.

### Changed
- Updated dependencies.
- Release workflow now adds a link to the latest version changes in `CHANGELOG.md` to the release notes.
- Refactored property resolution from profile settings.
- Updated `.gitignore` rules to ignore `config/*` except `config/config.toml` and allowed profile files (`config/profiles/dev*.toml`, `config/profiles/unit-tests.toml`).
- Release trigger filters now include `[release:patch]` and `[release:fix]` in `release.yml` as additional patch-level release tokens with the same behaviour as `[release]`.
- Enhanced `README.md` with GitHub workflows section.
- Heavily reworked `release.yml` and `Dockerfile` to speed up the release workflow.

### Fixed
- Average per day values in expense and transfer statistics was wrongly computed. This fix requires [database](database) schema updates. Run the following script to migrate:
  - [`005_fix_overwrite_stats_function.surql`](database/migrations/005_fix_overwrite_stats_function.surql)

## [0.2.4] - 2025-07-09
### 🔧 Patch Release
### Added
- `docker-compose.yml` template to easily run and update the deployment.
- Docker compose guide to `Docker Setup` section in [README.md](README.md).
- `dev-local-docker.toml` profile configurations to access localhost from within the Docker container.
- Log line on startup to show the loaded profile.
- Log line when an error occurs during database connection.

### Changed
- `Dockerfile` to enable caching.
- Translations for statistics now hide missing stats when count is `0`. This functionality requires [database](database) schema updates. Run the following script to migrate:
  - [`004_overwrite_traveler_stats_function.surql`](database/migrations/004_overwrite_traveler_stats_function.surql)

### Fixed
- `Database Setup` section in [README.md](README.md) was deleted by mistake.
- Migration script [`002_add_timestamps.surql`](database/migrations/002_add_timestamps.surql) left timestamps `NULL` for already existing records. Run the `UPDATE` statements to populate the missing data:
  ```sql
  UPDATE expense SET timestamp_utc = time::now() WHERE timestamp_utc IS NULL;
  UPDATE transferred_to SET timestamp_utc = time::now() WHERE timestamp_utc IS NULL;
  ```

## [0.2.3] - 2025-06-21
### 🔧 Patch Release
### Added
- Timestamps for expenses and transfers. This functionality requires [database](database) schema updates through migration scripts (view **Changed** section).
- `ShowStats` command to show some statistics about expenses, transfers and travelers. This functionality requires [database](database) schema updates through migration scripts (view **Changed** section).
- [Logo](assets/logo-dev.png) for DEV bot.

### Changed
- Updated [database](database) schema. Run the following scripts to migrate:
  - [`002_add_timestamps.surql`](database/migrations/002_add_timestamps.surql)
  - [`003_define_stats_functions.surql`](database/migrations/003_define_stats_functions.surql)
- Refactored the project structure to gather database scripts under one [directory](database).
- Minor changes to [locales](locales) files.
- Enhanced translations handling using `Translate` and `TranslateWithArgs` traits and defining indentation levels for lists of items.

### Fixed
- Minor debug logging formatting.
- ~~For the `rusty_money` crate, we now target [the git repository](https://github.com/varunsrin/rusty_money.git) to get the latest changes. This fixes some bugs present in the last version published on [crates.io](https://crates.io/crates/rusty-money) (`rusty-money = "0.4.1"` to date).~~

## [0.2.2] - 2025-06-08
### 🔧 Patch Release
### Added
- N/A

### Changed
- Enhanced release workflow with concurrency control.
- Added more target platforms for building Docker container images to release workflow.
- Restructured the release workflow to prevent partial releases from being created in case of build failures.

### Fixed
- Typo in italian translations file for `AddExpense` command.

## [0.2.1] - 2025-06-02
### 🔧 Patch Release
### Added
- Docker configuration.
- `build.rs` to copy configuration and locales folders into the build location.

### Changed
- Updated dependencies.
- Enhanced `README.md` with Docker setup instructions.
- Removed the `Generate Release Notes` step from `release.yml` in favor of GitHub built-in release notes generation.

### Fixed
- N/A

## [0.2.0] - 2025-05-27
### ✨ Minor Release
### Added
- GitHub CI configurations.

### Changed
- N/A

### Fixed
- N/A

## [0.1.11] - 2025-05-27
### Added
- Unit and integration tests.

### Changed
- Instead of using a `'static` database connection, connections are now stored in `Arc`s. This is necessary because each test uses a new in-memory connection.
- Logs are now written to a subfolder of the specified `logging.path`, named after the `profile` used.
- Translation for `ExpenseDetails` moved from `commands.ftl` to `format.ftl`.
- `types.rs` renamed to `format.rs`.

### Fixed
- Check existence of `Chat` before creating or updating the record in the database.
- Exceeding the total amount when adding an expense reported that shares had been cleared, but they were not. Now, shares are cleared correctly.
- Setting fixed shares that summed up to the total amount caused a division by zero.
- Fixed `clear_debts` DB function.
- If balances are all zero after rounding, calling `ShowBalances` command now correctly shows that travelers are all settled up.

## [0.1.10] - 2025-04-19
### Added
- N/A

### Changed
- N/A

### Fixed
- An issue where new dialogues were created when an unknown command was invoked.

## [0.1.9] - 2025-04-11
### Added
- Logo.

### Changed
- Updated `README.md`.
- Updated dependencies.
- Logs timestamps use UTC time.

### Fixed
- N/A

## [0.1.8] - 2025-03-30
### Added
- `ROADMAP.md` file.

### Changed
- `README.md` now contains an overall description of the project while the roadmap has been moved to a dedicated file.
- Error messages for the `Help` command are now more descriptive.
- The `DeleteTraveler` command now prevents deleting a traveler with associated expenses. A warning message is displayed, prompting the user to remove the related expenses first.

### Fixed
- Database indexes conflicting with each other.

## [0.1.7] - 2025-03-23
### Added
- Support for command line argument `profile`.
- `[logging]` section in (profile) settings files.

### Changed
- Security: set `.gitignore` to ignore uploading all configuration profiles except `dev-local.toml`.
- If command line argument `profile` is specified, the bot will use the specified profile settings instead of loading from `config.toml`.
- Logs are now written to files instead of `stdout`.

### Fixed
- N/A

## [0.1.6] - 2025-03-16
### Added
- `DeleteTransfer` and `ListTransfers` commands.

### Changed
- Merged `ShowBalance` command into `ShowBalances`:
  - Removed `ShowBalance` command.
  - Changed `ShowBalances` to accept an optional name.
- Merged `FindExpenses` command into `ListExpenses`:
  - Removed `FindExpenses` command.
  - Changed `ListExpenses` to accept an optional description.

### Fixed
- Not updating debts when deleting expenses.

## [0.1.5] - 2025-03-15
### Added
- Support for currency output formatting.
- Default currency configurable in (profile) settings files.
- `SetCurrency` command.

### Changed
- Sorted results from database queries.

### Fixed
- N/A

## [0.1.4] - 2025-03-09
### Added
- Support for i18n.
- `en-US`, `it-IT` locales.
- `[i18n]` section in (profile) settings files.
- `SetLanguage` command.

### Changed
- Improved settings: now supporting dedicated profile settings files.
- Bot replies are translated using lookup files.
- Updated dependencies.

### Fixed
- `unknown_command` didn't return the "Invalid usage" message when arguments were specified.

## [0.1.3] - 2025-02-25
### Added
- `ShowExpense` command.

### Changed
- Updated some help messages.

### Fixed
- An error returned passing `all` when asked to continue splitting an expense.

## [0.1.2] - 2025-02-22
### Added
- `ShowBalance` and `ShowBalances` commands.
- Dedicated help messages for all commands.
- Handle `unknown_command` when command name is right but arguments are incomplete.

### Changed
- Updated to `2024` edition.
- Updated dependencies.
- Improved unknown command and error handling.
- Refactored and improved the project structure.
- Refactored imports.

### Fixed
- N/A

## [0.1.1] - 2025-02-19
### Added
- `Transfer` command.
- Calculation and simplification of debts.
- Added `CHANGELOG.md`.

### Changed
- Updated dependencies.
- Improved unknown command handling.
- Improved configurations management.
- Refactored and improved the project structure.
- Minor formatting changes.

### Fixed
- Minor fixes.
