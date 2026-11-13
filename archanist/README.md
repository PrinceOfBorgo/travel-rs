# Managing Travel-RS with Archanist

[Archanist](https://github.com/PrinceOfBorgo/archanist) is the small, dockerized update engine that keeps a Travel-RS deployment current. This directory holds everything needed to run it against Travel-RS:

```
archanist/
├── config.toml            # Archanist main config
├── docker-compose.yml     # runs Archanist as a container (drives the host docker)
└── components/
    └── travel-rs.toml     # recipe: upgrade the bot (download → migrate → swap)
```

The entire `archanist/` directory is shipped inside every Travel-RS release bundle (`deploy-v<version>.zip`, under `archanist/`), so you can bootstrap from the release zip alone. Two categories of files ship there:

- **Synced on every update** - `config.toml` + `components/`. The `travel-rs.toml` recipe copies these back over this instance on every update, so once Archanist is running you keep its automation current **just by releasing the bot** - no manual edits on the host.
- **First-time bootstrap only** - `docker-compose.yml` + `.env.example`. These seed the initial install and are **not** synced afterwards; edits you make to your live `docker-compose.yml` / `.env` survive updates untouched. The compose file is kept generic (`env_file: .env`, `pull_policy: always`) so releases normally only need new `.env` entries. When a release does need a host-side change - a `docker-compose.yml` edit, a new `.env` key, or deleting a recipe the bundle no longer ships (updates never delete files) - it is listed in [DEPLOYMENT.md, section 7](../DEPLOYMENT.md#7-manual-archanist-host-changes); apply it before running the update.

---

## What a `travel-rs` update does

Running `archanist update travel-rs` performs the automated equivalent of the manual procedure in [DEPLOYMENT.md](../DEPLOYMENT.md):

1. Check that every `.env` entry the recipe needs is set (fails before anything else runs).
2. Download `deploy-v<version>.zip` from the GitHub release.
3. Extract it (`unzip`).
4. Install Archanist's own `components/` from the bundle (each recipe is validated first) and continue the run with the new `travel-rs` recipe, so the remaining steps are the ones shipped with the release being deployed.
5. Read the bundle's `DEPLOYMENT.md`, select the migrations introduced **after** the currently-deployed version, and apply them via an ephemeral `surrealdb/surrealdb` container.
6. Copy the new locale files into the install dir.
7. Merge new keys from `prod.toml.example` into the active profile.
8. Merge the bundle's `config.toml` into Archanist's main config.
9. Pull the new bot image and recreate the `travel-rs` container.

Every step is recorded in `state.toml`; a failed run can be retried, and `archanist rollback travel-rs` undoes the completed steps in reverse.

---

## Image requirements

The `travel-rs` recipe runs its tools - `unzip` and the SurrealDB CLI - in throwaway `busybox` and `surrealdb/surrealdb` containers that Archanist pulls and drives through the mounted docker socket. **The Archanist image therefore needs nothing but its own binary and access to that socket** - no docker CLI, no unzip, no shell.

---

## First-time bootstrap

If Archanist has not been deployed yet, the very first update is seeded by hand. After that everything flows from Travel-RS releases.

1. **Copy this directory to the host**, next to where you want the data to live, e.g. `/example/path/to/travel-rs/archanist/`. You can take it either from this repo or from the `archanist/` folder inside a release `deploy-v<version>.zip`.

2. **Pin the Archanist image (optional).** `docker-compose.yml` defaults to `ghcr.io/princeofborgo/archanist:latest` and pulls it on every run (`pull_policy: always`), so Archanist updates itself just by being run. To pin a specific version instead, set `ARCHANIST_TAG` (and optionally `ARCHANIST_IMAGE`) in an `.env` file beside `docker-compose.yml`; you then upgrade Archanist by bumping that tag, and roll it back by setting it back.

3. **Set the host data path.** `docker-compose.yml` bind-mounts `./archanist-data` at `/app/data`. Put the ABSOLUTE host path that resolves to in `.env` as `ARCHANIST_HOST_DATA_DIR` (see step 5). The recipes read it from there via compose, so `sync_archanist_recipes` can overwrite the checked-in `components/*.toml` on every update without wiping your per-host value.

4. **Seed the current install into `archanist-data/`** so Archanist knows what is already deployed:

   ```
   archanist-data/
   ├── config.toml                         # copy of this dir's config.toml
   ├── components/                         # copy of this dir's components/
   ├── travel-rs/
   │   ├── config/  (config.toml, profiles/prod.toml)   # your live bot config
   │   └── locales/                        # your live locales
   └── state.toml                          # tells Archanist the deployed version
   ```

   Minimal `state.toml` for an existing deployment:

   ```toml
   [components.travel-rs]
   current_version = "<installed-version>"    # the version currently running on the host
   ```

   That is all you need to seed: the `migrate` step selects only the migrations listed *after* `current_version` in the bundle's Migration Reference table, so migrations at or below the installed version are never re-run.

5. **Provide the deployment settings.** The recipe reads the host data path, the active bot profile, and the SurrealDB connection from environment variables. Copy the tracked template and fill in your values:

   ```bash
   cp .env.example .env
   # then edit .env
   ```

   Keys (see [`.env.example`](.env.example) for the full template):

   ```dotenv
   ARCHANIST_HOST_DATA_DIR=/example/path/to/travel-rs/archanist/archanist-data  # absolute host path of ./archanist-data
   TRAVELRS_PROFILE=your-profile                                                # file name under config/profiles/, without .toml
   TRAVELRS_BOT_TOKEN=your-telegram-bot-token                                   # injected into the bot as the profile's token

   TRAVELRS_DB_ADDRESS=https://your-surreal-endpoint                            # http:// or https:// ONLY (see note below)
   TRAVELRS_DB_USERNAME=your-db-user
   TRAVELRS_DB_PASSWORD=your-db-password
   TRAVELRS_DB_NAMESPACE=travel_rs
   TRAVELRS_DB_NAME=travel_rs_db
   ```

   > **`TRAVELRS_DB_ADDRESS` must use `http://` or `https://`, not `ws://`/`wss://`.** The `migrate` step runs `surreal export`/`import`, and SurrealDB does not support those over WebSocket. SurrealDB serves HTTP and WS on the same port, so the http(s) scheme works for both the migration CLI and the bot (which uses the transport-agnostic `any` engine).

   `TRAVELRS_DB_*` must match the active profile's `[database]`. Compose passes every `.env` entry to Archanist (`env_file: .env`), which injects them both into the `migrate` step and into the recreated bot container. The update's first step fails if any required value is missing, before anything is downloaded or changed.

6. **Run it:**

   ```bash
   cd /example/path/to/travel-rs/archanist
   docker compose run --rm archanist status      # sanity check
   docker compose run --rm archanist update      # upgrade the bot
   ```

From the next release onward, step 4's `components/` and `config.toml` inside `archanist-data/` are overwritten automatically by the update, and the Archanist image is refreshed by docker compose on every run.

---

## Everyday operation

```bash
cd /example/path/to/travel-rs/archanist

docker compose run --rm archanist check          # what updates are available
docker compose run --rm archanist update         # update all components (just travel-rs)
docker compose run --rm archanist update travel-rs  # only travel-rs, even if stale recipes linger in components/
docker compose run --rm archanist rollback travel-rs  # component name is required
docker compose run --rm archanist status
```

Every `docker compose run` first pulls the Archanist image (`pull_policy: always`), so commands always run on the newest Archanist, or on `ARCHANIST_TAG` if you pinned it in `.env`. `update` therefore only has the bot to update.

## Rehearsing an update (sandbox)

The `[sandbox]` section of `components/travel-rs.toml` lets Archanist rehearse the bot upgrade end to end without touching production: it starts an in-memory SurrealDB (`travel-rs-sandbox-surrealdb`) and a v0.2.5 placeholder (`travel-rs-sandbox`) on the `archanist-sandbox` network, lays down a v0.2.5 baseline (schema + migrations `001..005`, locales, profile) once, then runs the real update against the latest release. All files land under `archanist-data/sandbox/travel-rs/`, with its own `state.toml`.

```bash
docker compose run --rm archanist sandbox run travel-rs       # rehearse (re-runs skip the baseline)
docker compose run --rm archanist sandbox rollback travel-rs  # test rollback
docker compose run --rm archanist sandbox reset travel-rs     # wipe containers, network, sandbox dir
```

The upgraded sandbox bot exits right away (dummy token); the rehearsal checks the pipeline - migrations applied, image swapped - not a running bot.

The `archanist-sandbox.yml` GitHub workflow runs this rehearsal automatically after every release, using the recipes shipped in that release's bundle.
