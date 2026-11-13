<!-- omit from toc -->
# Deployment Guide

This document covers production deployment, upgrades, and database migration procedures for Travel-RS Bot.

> **Default method: Archanist.** The recommended way to install and keep Travel-RS up to date is [Archanist](https://github.com/PrinceOfBorgo/archanist), the dockerized update engine shipped in every release bundle under `archanist/`. It automates the upgrade procedure below (download, migrations, locales, profile merge, container swap) with state tracking and rollback. **Follow the [Archanist guide](archanist/README.md)** for bootstrap and everyday operation, and check [section 7 (Manual Archanist Host Changes)](#7-manual-archanist-host-changes) before each update.
>
> The manual procedures in sections 2-4 (Docker or standalone binary) remain documented as a fallback and as a reference for what Archanist does under the hood.

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
- [7. Manual Archanist Host Changes](#7-manual-archanist-host-changes)
  - [7.1. v0.4.5](#71-v045)
  - [7.2. v0.4.3](#72-v043)

## 1. Prerequisites

- Access to the [GitHub Releases](https://github.com/PrinceOfBorgo/travel-rs/releases) page.
- A SurrealDB instance (cloud or self-hosted) accessible from the deployment host.
- A valid Telegram bot token (obtain one from [@BotFather](https://t.me/BotFather)).

**For Archanist deployment (default) and manual Docker deployment:**
- **Docker Desktop** (Windows/macOS) or **Docker Engine** (Linux) installed and running.

**For manual deployment:**
- A `travel-rs` binary for your platform, built from source with `cargo build --release`. The release workflow only publishes Docker images — standalone binaries are not provided as release assets.

## 2. Fresh Installation

> **Using Archanist (default)?** Follow the [Archanist guide](archanist/README.md#first-time-bootstrap) instead. On a fresh install Archanist applies every migration automatically. The steps below are the manual equivalent.

### 2.1. Download the Deploy Bundle

Go to the [latest GitHub Release](https://github.com/PrinceOfBorgo/travel-rs/releases/latest) and download the **`deploy-v<version>.zip`** asset. Extract it to your desired deployment directory:

```bash
mkdir -p /example/path/to/travel-rs && cd /example/path/to/travel-rs
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
   - Bot token source and value (file path or direct string).
   - Database connection details (address, username, password, namespace, database).
   - Logging preferences.
   - Default locale and currency.

   > Every field value can also contain environment variables `${VAR_NAME}` that will be resolved at runtime. Referencing an undefined variable causes the bot to fail fast at startup with an error identifying the offending property path.

3. **Provide the bot token:**

   Depending on your `token_source` setting, either:
   - Create a file (e.g., `config/prod-token.txt`) containing only the token, or
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

> Manual Docker deployment. Prefer [Archanist](archanist/README.md), which manages the bot container for you.

```bash
docker compose up -d
```

Check that the container is running:

```bash
docker compose ps
docker compose logs -f travel-rs
```

### 2.5. Start the Bot (Manual)

> [Archanist](archanist/README.md) is the default deployment method, and plain Docker (see [the previous section](#24-start-the-bot-docker)) is the manual alternative. Use this section only if Docker is not available in your environment.

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
   /example/path/to/travel-rs/
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
   cd /example/path/to/travel-rs
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
   WorkingDirectory=/example/path/to/travel-rs
   ExecStart=/example/path/to/travel-rs/travel-rs
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

> **Using Archanist (default)?** Apply any host changes listed in [section 7](#7-manual-archanist-host-changes) for the target release, then run:
>
> ```bash
> cd /example/path/to/travel-rs/archanist
> docker compose run --rm archanist update travel-rs
> ```
>
> See the [Archanist guide](archanist/README.md#everyday-operation) for details. The steps below are the manual equivalent.

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

> [Archanist](archanist/README.md) is the default deployment method, and plain Docker (see [the previous section](#36-update-and-restart-docker)) is the manual alternative. Use this section only if you are running the binary directly.

If running the binary directly:

1. **Stop the current process** (or stop the systemd service):

   ```bash
   sudo systemctl stop travel-rs
   ```

2. **Replace the binary** with the new version built from source:

   ```bash
   # Build the new version (see section 2.5 step 1)
   cp target/release/travel-rs /example/path/to/travel-rs/travel-rs
   chmod +x /example/path/to/travel-rs/travel-rs
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

> **Using Archanist (default)?** Run `docker compose run --rm archanist rollback travel-rs` from the `archanist/` directory. It undoes the completed update steps in reverse, including restoring the database snapshot taken before migrating. See the [Archanist guide](archanist/README.md#everyday-operation).

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
├── config/
│   ├── config.toml                      # Main config (set your profile here)
│   └── profiles/
│       └── prod.toml.example            # Sanitized profile template
└── archanist/                           # Archanist update-engine automation (see archanist/README.md)
    ├── config.toml                      # Archanist main config (synced on every update)
    ├── docker-compose.yml               # First-time bootstrap only (not synced, see section 7)
    ├── .env.example                     # First-time bootstrap template (not synced, see section 7)
    ├── README.md                        # Archanist bootstrap & usage guide (not synced)
    └── components/
        └── travel-rs.toml               # Recipe: upgrade the bot (synced on every update)
```

## 6. Migration Reference

The following table maps each release to the database migration scripts it introduced. When upgrading, run only the scripts that are new since your currently deployed version.

> **For contributors:** If your release introduces new database migration scripts (referenced in `CHANGELOG.md`), you **must** add a corresponding row to this table before pushing the release commit. The release workflow validates this and will fail if the entry is missing.
>
> Row format: ``| v<version> | `001_script_name.surql`, `002_other_script.surql` | Brief description |``

| Version | Required Migrations                                                                 | Notes                                       |
| ------- | ----------------------------------------------------------------------------------- | ------------------------------------------- |
| v0.2.0  | `001_init.surql`                                                                    | Initial schema                              |
| v0.2.3  | ~~`002_add_timestamps.surql`~~, `003_define_stats_functions.surql`                  | Timestamps and statistics functions         |
| v0.2.4  | `002_add_timestamps.surql`, `004_overwrite_traveler_stats_function.surql`           | Updated stats function; timestamps fix      |
| v0.2.5  | `005_fix_overwrite_stats_function.surql`                                            | Fix average per day stats                   |
| v0.3.0  | `006_add_validation_constraints.surql`, `007_case_insensitive_traveler_names.surql` | Schema validation constraints               |
| v0.3.1  | `008_add_traveler_number.surql`                                                     | Stable numeric ID for travelers             |
| v0.3.2  | `009_assert_chat_equality.surql`                                                    | Chat-equality assertions on relation tables |

## 7. Manual Archanist Host Changes

An [Archanist](archanist/README.md) update keeps the bot, its locales, its profile, and Archanist's own `config.toml` + `components/` current. It does **not** touch the host files you own, and it never deletes files:

- `archanist/docker-compose.yml` and `archanist/.env` are not synced.
- A recipe dropped from the bundle stays in `archanist-data/components/` until you delete it.

When a release needs one of those changes, it is listed below. **Apply it before running `docker compose run --rm archanist update`**. If you skip releases, apply every entry from your deployed version up to the target, oldest first.

> **For contributors:** if a release changes `archanist/docker-compose.yml`, adds a required key to `archanist/.env.example`, or removes a file from the bundle's `archanist/` directory, add an entry here before releasing.

### 7.1. v0.4.5

Archanist is now updated by docker compose (it pulls the newest image on every run) instead of by an `archanist` recipe, and the compose file passes the whole `.env` to Archanist.

1. **Update `archanist/docker-compose.yml`.** Replace it with the bundle's `archanist/docker-compose.yml`, or edit the `archanist` service by hand:
   - add `pull_policy: always`;
   - add `env_file: .env`;
   - remove the `environment:` list (every key now comes from `.env`);
   - remove `restart: unless-stopped`.
2. **Delete the retired recipe** so `update` stops processing it:

   ```bash
   rm archanist-data/components/archanist.toml
   ```

3. **Remove the leftover Archanist container**, if one exists. The old recipe recreated a long-lived `archanist` container that only prints `--help` and restarts forever:

   ```bash
   docker rm -f archanist
   ```

4. *(Optional)* Remove the `self_component = "archanist"` line from `archanist-data/config.toml`. The config sync never removes keys, but a leftover value is harmless.
5. **Check** that the new image is pulled:

   ```bash
   docker compose run --rm archanist --version
   ```

   You don't need to check `.env` by hand: the first step of every update (`check_env`) fails before anything is downloaded if a required key is missing. A failed update blocklists the target version, so after fixing `.env` run `docker compose run --rm archanist unblock travel-rs <version>`.

### 7.2. v0.4.3

The bot reads its token from the `TRAVELRS_BOT_TOKEN` environment variable instead of `config/prod-token.txt`; the `swap` step injects it into the bot container.

1. Add `TRAVELRS_BOT_TOKEN=<your token>` to `archanist/.env`.
2. Only if you stay below v0.4.5: in `archanist/docker-compose.yml`, add `TRAVELRS_BOT_TOKEN: ${TRAVELRS_BOT_TOKEN:?set TRAVELRS_BOT_TOKEN in .env}` under the `archanist` service's `environment:`. From v0.4.5 the whole `.env` is passed through.

If the token never reaches Archanist, the bot container is recreated without it and restarts in a loop with `Environment variable 'TRAVELRS_BOT_TOKEN' for bot.token should be set`.
