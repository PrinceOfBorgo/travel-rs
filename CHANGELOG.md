# Changelog

## [0.2.3-SNAPSHOT] - Unreleased
### Added
- Timestamps for expenses and transfers. This functionality requires [database](database) schema updates through migration scripts:
  - [`002_add_timestamps.surql`](database/migrations/002_add_timestamps.surql)
- [Logo](assets/logo-dev.png) for DEV bot.

### Changed
- Refactored the project structure to gather database scripts under one [directory](database).
- Minor changes to [locales](locales) files.
- For the `rusty_money` crate, we now target [the git repository](https://github.com/varunsrin/rusty_money.git) to get the latest changes. This fixes some bugs present in the last version published on [crates.io](https://crates.io/crates/rusty-money) (`rusty-money = "0.4.1"` to date).

### Fixed
- N/A.

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
