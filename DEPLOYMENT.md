<!-- omit from toc -->
# Deployment Guide

This document covers production deployment, upgrades, and database migration procedures for Travel-RS Bot. The recommended deployment method is **Docker**; a manual (binary) deployment path is also documented for environments where Docker is not available.

For general Docker setup, volume configuration, and running the container, see the [Docker Setup](README.md#10-docker-setup) section in the README.

<!-- omit from toc -->
## Table of Contents
- [1. Prerequisites](#1-prerequisites)
- [2. Fresh Installation](#2-fresh-installation)
  - [2.1. Download the Deploy Bundle](#21-download-the-deploy-bundle)
  - [2.2. Configure the Bot](#22-configure-the-bot)
  - [2.3. Initialize the Database](#23-initialize-the-database)
  - [2.4. Start the Bot (Docker)](#24-start-the-bot-docker)
  - [2.5. Start the Bot (Manual)](#25-start-the-bot-manual)
- [3. Upgrading to a New Version](#3-upgrading-to-a-new-version)
  - [3.1. Download the New Deploy Bundle](#31-download-the-new-deploy-bundle)
  - [3.2. Back Up Current State](#32-back-up-current-state)
  - [3.3. Update Locale Files](#33-update-locale-files)
  - [3.4. Run Database Migrations](#34-run-database-migrations)
  - [3.5. Review Configuration Changes](#35-review-configuration-changes)
  - [3.6. Update and Restart (Docker)](#36-update-and-restart-docker)
  - [3.7. Update and Restart (Manual)](#37-update-and-restart-manual)
  - [3.8. Verify the Upgrade](#38-verify-the-upgrade)
  - [3.9. Cleanup](#39-cleanup)
- [4. Rollback](#4-rollback)
  - [4.1. Docker Rollback](#41-docker-rollback)
  - [4.2. Manual Rollback](#42-manual-rollback)
- [5. Deploy Bundle Contents](#5-deploy-bundle-contents)
- [6. Migration Reference](#6-migration-reference)

## 1. Prerequisites

- Access to the [GitHub Releases](https://github.com/PrinceOfBorgo/travel-rs/releases) page.
- A SurrealDB instance (cloud or self-hosted) accessible from the deployment host.
- A valid Telegram bot token (obtain one from [@BotFather](https://t.me/BotFather)).

**For Docker deployment (recommended):**
- **Docker Desktop** (Windows/macOS) or **Docker Engine** (Linux) installed and running.

**For manual deployment:**
- A `travel-rs` binary for your platform, built from source with `cargo build --release`. The release workflow only publishes Docker images — standalone binaries are not provided as release assets.

## 2. Fresh Installation

### 2.1. Download the Deploy Bundle

Go to the [latest GitHub Release](https://github.com/PrinceOfBorgo/travel-rs/releases/latest) and download the **`deploy-v<version>.zip`** asset. Extract it to your desired deployment directory:

```bash
mkdir -p /opt/travel-rs && cd /opt/travel-rs
unzip deploy-v<version>.zip
```

The extracted bundle contains everything needed to run the bot. See [Deploy Bundle Contents](#5-deploy-bundle-contents) for details.

### 2.2. Configure the Bot

1. **Set the active profile:**

   Edit `config/config.toml` and set the profile to use:

   ```toml
   profile = "prod"
   ```

2. **Create or edit your profile:**

   Copy the provided example and fill in your actual values:

   ```bash
   cp config/profiles/prod.toml.example config/profiles/prod.toml
   ```

   Edit `config/profiles/prod.toml` to set:
   - Bot token source and value (file path, environment variable, or direct string).
   - Database connection details (address, username, password, namespace, database).
   - Logging preferences.
   - Default locale and currency.

3. **Provide the bot token:**

   Depending on your `token_source` setting, either:
   - Create a file (e.g., `config/prod-token.txt`) containing only the token, or
   - Set an environment variable before starting the container, or
   - Embed the token directly as a string in the profile (not recommended — the token will be stored in plain text in the configuration file).

### 2.3. Initialize the Database

For a **new** database, run the full schema build script against your SurrealDB instance:

```bash
surreal import --conn <DB_ADDRESS> --user <USER> --pass <PASS> \
  --ns <NAMESPACE> --db <DATABASE> \
  database/build_travelers_db.surql
```

Then apply all migration scripts in order:

```bash
for f in database/migrations/*.surql; do
  surreal import --conn <DB_ADDRESS> --user <USER> --pass <PASS> \
    --ns <NAMESPACE> --db <DATABASE> "$f"
done
```

### 2.4. Start the Bot (Docker)

```bash
docker compose up -d
```

Check that the container is running:

```bash
docker compose ps
docker compose logs -f travel-rs
```

### 2.5. Start the Bot (Manual)

> Docker is the recommended deployment method (see [the previous section](#24-start-the-bot-docker)). Use this section only if Docker is not available in your environment.

If you prefer to run the binary directly without Docker:

1. **Build the binary** from source:

   ```bash
   git clone https://github.com/PrinceOfBorgo/travel-rs.git
   cd travel-rs
   git checkout v<version>   # check out the target release tag
   cargo build --release
   # Binary will be at target/release/travel-rs
   ```

2. **Place the binary** in your deployment directory alongside the `config/` and `locales/` directories:

   ```
   /opt/travel-rs/
   ├── travel-rs              # the binary
   ├── config/
   │   ├── config.toml
   │   └── profiles/
   │       └── prod.toml
   └── locales/
       ├── en-US/
       └── it-IT/
   ```

3. **Run the bot:**

   ```bash
   cd /opt/travel-rs
   ./travel-rs
   ```

   Or with a specific profile override:

   ```bash
   ./travel-rs --profile prod
   ```

4. **(Optional) Run as a systemd service** for automatic restarts and log management:

   ```ini
   # /etc/systemd/system/travel-rs.service
   [Unit]
   Description=Travel-RS Telegram Bot
   After=network.target

   [Service]
   Type=simple
   WorkingDirectory=/opt/travel-rs
   ExecStart=/opt/travel-rs/travel-rs
   Restart=on-failure
   RestartSec=5
   Environment=RUST_LOG=info

   [Install]
   WantedBy=multi-user.target
   ```

   Enable and start:

   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now travel-rs
   ```

   View logs:

   ```bash
   sudo journalctl -u travel-rs -f
   ```

## 3. Upgrading to a New Version

### 3.1. Download the New Deploy Bundle

Download the **`deploy-v<version>.zip`** from the target [GitHub Release](https://github.com/PrinceOfBorgo/travel-rs/releases) and extract it to a temporary location:

```bash
unzip deploy-v<version>.zip -d /tmp/travel-rs-upgrade
```

### 3.2. Back Up Current State

Before making any changes, back up your current deployment:

```bash
# Back up configuration (contains credentials)
cp -r config config.bak

# Back up locale files
cp -r locales locales.bak

# Back up the database (if self-hosted)
# Use your SurrealDB backup method of choice
```

### 3.3. Update Locale Files

Replace the locale files with the ones from the new deploy bundle:

```bash
rm -rf locales/*
cp -r /tmp/travel-rs-upgrade/locales/* locales/
```

> **Note:** Locale files must always match the bot version. Using outdated locale files may result in missing translations or errors.

### 3.4. Run Database Migrations

Check `MIGRATIONS.md` inside the deploy bundle to see which migration scripts are **new** for this release. Only run the scripts you haven't applied yet:

```bash
# Example: if upgrading from v0.2.5 to v0.3.0, run migrations 006 and 007
surreal import --conn <DB_ADDRESS> --user <USER> --pass <PASS> \
  --ns <NAMESPACE> --db <DATABASE> \
  database/migrations/006_add_validation_constraints.surql

surreal import --conn <DB_ADDRESS> --user <USER> --pass <PASS> \
  --ns <NAMESPACE> --db <DATABASE> \
  database/migrations/007_case_insensitive_traveler_names.surql
```

> **Tip:** Migration scripts are numbered sequentially and are idempotent in their definitions. If unsure which ones you've already applied, check the [Migration Reference](#6-migration-reference) table below.

### 3.5. Review Configuration Changes

Compare the new example profile with your current one to check for new or changed settings:

```bash
diff config/profiles/prod.toml config/profiles/prod.toml.example
```

Update your profile as needed. Your existing `config.toml` and bot token typically don't change between versions.

### 3.6. Update and Restart (Docker)

Replace the `docker-compose.yml` with the version-pinned one from the bundle, then pull and restart:

```bash
cp /tmp/travel-rs-upgrade/docker-compose.yml docker-compose.yml
docker compose pull travel-rs
docker compose up -d
```

### 3.7. Update and Restart (Manual)

> Docker is the recommended deployment method (see [the previous section](#36-update-and-restart-docker)). Use this section only if you are running the binary directly.

If running the binary directly:

1. **Stop the current process** (or stop the systemd service):

   ```bash
   sudo systemctl stop travel-rs
   ```

2. **Replace the binary** with the new version built from source:

   ```bash
   # Build the new version (see section 2.5 step 1)
   cp target/release/travel-rs /opt/travel-rs/travel-rs
   chmod +x /opt/travel-rs/travel-rs
   ```

3. **Restart:**

   ```bash
   sudo systemctl start travel-rs
   ```

### 3.8. Verify the Upgrade

```bash
# Check container is running
docker compose ps

# Check logs for startup errors
docker compose logs -f travel-rs
```

Send a `/help` command to the bot in Telegram to confirm it's responding correctly.

### 3.9. Cleanup

```bash
rm -rf /tmp/travel-rs-upgrade
rm -rf config.bak locales.bak  # after confirming everything works
```

## 4. Rollback

If an upgrade fails:

### 4.1. Docker Rollback

1. **Stop the current container:**

   ```bash
   docker compose down
   ```

2. **Restore backups:**

   ```bash
   cp -r config.bak/* config/
   cp -r locales.bak/* locales/
   ```

3. **Revert to the previous image version:**

   Edit `docker-compose.yml` to point to the previous version tag:

   ```yaml
   image: ghcr.io/princeofborgo/travel-rs:v<previous-version>
   ```

4. **Restart:**

   ```bash
   docker compose up -d
   ```

### 4.2. Manual Rollback

> If you deployed with Docker, see [the previous section](#41-docker-rollback) instead.

1. **Stop the bot:**

   ```bash
   sudo systemctl stop travel-rs
   ```

2. **Restore backups:**

   ```bash
   cp -r config.bak/* config/
   cp -r locales.bak/* locales/
   ```

3. **Replace the binary** with the previous version (keep a backup of the binary before upgrading, or rebuild the older version from source):

   ```bash
   cp travel-rs.bak travel-rs
   ```

4. **Restart:**

   ```bash
   sudo systemctl start travel-rs
   ```

> **Note:** Database migrations cannot be automatically rolled back. If a migration caused issues, you may need to manually revert the schema changes or restore from a database backup.

## 5. Deploy Bundle Contents

Each release includes a `deploy-v<version>.zip` asset with the following structure:

```
deploy-v<version>/
├── docker-compose.yml                   # Pinned to the release version tag
├── DEPLOYMENT.md                        # This deployment guide
├── MIGRATIONS.md                        # Migration scripts required for this release
├── locales/                             # Up-to-date locale files
│   ├── en-US/
│   │   ├── commands.ftl
│   │   ├── errors.ftl
│   │   ├── format.ftl
│   │   ├── help.ftl
│   │   ├── labels.ftl
│   │   └── dialogues/
│   └── it-IT/
│       └── ...
├── database/
│   ├── build_travelers_db.surql         # Full schema build (for fresh installs)
│   └── migrations/                      # All migration scripts (apply in order)
│       ├── 001_init.surql
│       ├── 002_add_timestamps.surql
│       └── ...
└── config/
    ├── config.toml                      # Main config (set your profile here)
    └── profiles/
        └── prod.toml.example            # Sanitized profile template
```

## 6. Migration Reference

The following table maps each release to the database migration scripts it introduced. When upgrading, run only the scripts that are new since your currently deployed version.

> **For contributors:** If your release introduces new database migration scripts (referenced in `CHANGELOG.md`), you **must** add a corresponding row to this table before pushing the release commit. The release workflow validates this and will fail if the entry is missing.
>
> Row format: `| v<version> | `001_script_name.surql`, `002_other_script.surql` | Brief description |`

| Version | Required Migrations                                                                 | Notes                         |
| ------- | ----------------------------------------------------------------------------------- | ----------------------------- |
| v0.2.0  | `001_init.surql`                                                                    | Initial schema                |
| v0.2.1  | `002_add_timestamps.surql`                                                          | Adds timestamp fields         |
| v0.2.2  | `003_define_stats_functions.surql`                                                  | Statistics functions          |
| v0.2.4  | `004_overwrite_traveler_stats_function.surql`                                       | Updated stats function        |
| v0.2.5  | `005_fix_overwrite_stats_function.surql`                                            | Fix average per day stats     |
| v0.3.0  | `006_add_validation_constraints.surql`, `007_case_insensitive_traveler_names.surql` | Schema validation constraints |
