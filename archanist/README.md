# Managing Travel-RS with Archanist

[Archanist](https://github.com/PrinceOfBorgo/archanist) is the small, dockerized update engine that keeps a Travel-RS deployment current. This directory holds everything needed to run it against Travel-RS:

```
archanist/
├── config.toml            # Archanist main config (self_component = archanist)
├── docker-compose.yml     # runs Archanist as a container (drives the host docker)
└── components/
    ├── archanist.toml     # recipe: self-update the Archanist container
    └── travel-rs.toml     # recipe: upgrade the bot (download → migrate → swap)
```

The entire `archanist/` directory is shipped inside every Travel-RS release bundle (`deploy-v<version>.zip`, under `archanist/`), so you can bootstrap from the release zip alone. Two categories of files ship there:

- **Synced on every update** - `config.toml` + `components/`. The `travel-rs.toml` recipe copies these back over this instance on every update, so once Archanist is running you keep its automation current **just by releasing the bot** - no manual edits on the host.
- **First-time bootstrap only** - `docker-compose.yml` + `.env.example`. These seed the initial install and are **not** synced afterwards; edits you make to your live `docker-compose.yml` / `.env` survive updates untouched. When a release changes `docker-compose.yml`, apply the change by hand (it will be called out in the bundle's `MIGRATIONS.md`).

---

## What a `travel-rs` update does

Running `archanist update --component travel-rs` performs the automated equivalent of the manual procedure in [DEPLOYMENT.md](../DEPLOYMENT.md):

1. Download `deploy-v<version>.zip` from the GitHub release.
2. Extract it (`unzip`).
3. Read the bundle's `DEPLOYMENT.md`, select the migrations introduced **after** the currently-deployed version, and apply them via an ephemeral `surrealdb/surrealdb` container.
4. Copy the new locale files into the install dir.
5. Merge new keys from `prod.toml.example` into the active profile.
6. Refresh Archanist's own `components/` + `config.toml` from the bundle.
7. Pull the new bot image and recreate the `travel-rs` container.

Every step is recorded in `state.toml`; a failed run can be retried, and `archanist rollback travel-rs` undoes the completed steps in reverse.

---

## Image requirements

The `travel-rs` recipe runs its tools - `unzip` and the SurrealDB CLI - in throwaway `busybox` and `surrealdb/surrealdb` containers that Archanist pulls and drives through the mounted docker socket. **The Archanist image therefore needs nothing but its own binary and access to that socket** - no docker CLI, no unzip, no shell.

---

## First-time bootstrap

If Archanist has not been deployed yet, the very first update is seeded by hand. After that everything flows from Travel-RS releases.

1. **Copy this directory to the host**, next to where you want the data to live, e.g. `/example/path/to/travel-rs/archanist/`. You can take it either from this repo or from the `archanist/` folder inside a release `deploy-v<version>.zip`.

2. **Pin the Archanist image (optional).** `docker-compose.yml` defaults to `ghcr.io/princeofborgo/archanist:latest`, pulled from GHCR on first run. To pin a specific version, set `ARCHANIST_TAG` (and optionally `ARCHANIST_IMAGE`) in an `.env` file beside `docker-compose.yml`.

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
   # TRAVELRS_PROFILE=your-profile

   TRAVELRS_DB_ADDRESS=wss://your-surreal-endpoint                              # ws:// wss:// http:// or https://
   TRAVELRS_DB_USERNAME=your-db-user
   TRAVELRS_DB_PASSWORD=your-db-password
   TRAVELRS_DB_NAMESPACE=travel_rs
   TRAVELRS_DB_NAME=travel_rs_db
   ```

   `TRAVELRS_DB_*` must match the active profile's `[database]`. Compose refuses to start if any required value is missing, and Archanist injects them both into the `migrate` step and into the recreated bot container.

6. **Run it:**

   ```bash
   cd /example/path/to/travel-rs/archanist
   docker compose run --rm archanist status      # sanity check
   docker compose run --rm archanist update      # upgrade bot + Archanist
   ```

From the next release onward, step 4's `components/` and `config.toml` inside `archanist-data/` are overwritten automatically by the update, and the Archanist container is refreshed by its own `archanist` recipe.

---

## Everyday operation

```bash
cd /example/path/to/travel-rs/archanist

docker compose run --rm archanist check          # what updates are available
docker compose run --rm archanist update         # update all components
docker compose run --rm archanist update --component travel-rs
docker compose run --rm archanist rollback travel-rs
docker compose run --rm archanist status
```

`update` with no component updates every configured component: `travel-rs` (the bot) and `archanist` (itself). The self-update swaps the Archanist container and exits; the compose `restart: unless-stopped` policy brings the new image back up.
