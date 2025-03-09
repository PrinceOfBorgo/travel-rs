# Changelog

## [0.1.4-SNAPSHOT]
### Added
- Support for i18n.
- `en-US` locale.
- `[i18n]` section in (profile) settings files.

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
