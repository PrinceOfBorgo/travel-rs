# Changelog

## [0.1.10]
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
