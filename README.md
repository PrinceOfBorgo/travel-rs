<div align="center">
    <img src="assets/logo.svg" width="256"/>
</div>

# Travel-RS Bot

[![Github Link][github badge]][github link]
[![Cargo Build & Test][build & test badge]][build & test link]
[![Release][release badge]][release link]
[![Codecov][codecov badge]][codecov link]

Travel-RS Bot is a Rust-based Telegram bot designed to assist with managing travel-related expenses, debts, and balances. It provides a seamless experience for tracking financial transactions among travelers, offering localization support, and enabling flexible configurations.

The primary goal of Travel-RS Bot is to simplify the management of shared expenses during group travel. It helps users keep track of who owes what, manage repayments, and calculate balances efficiently. The bot is designed to be robust, configurable, and user-friendly.

## Adding Travel-RS Bot to a Telegram Group Chat

To use Travel-RS Bot in a Telegram group chat:

1. Create a new group chat or open an existing one.
2. Search for `@TravelRS_bot` and add it to the group.
3. Grant it admin permissions.

Once added and configured, the bot will be ready to assist with managing travel-related expenses.

## Key Functionalities

* **Expense Management**: Add, view, and delete expenses with detailed descriptions.
* **Debt Tracking**: Track debts between travelers and manage repayments.
* **Balance Calculation**: Simplify balances to minimize the number of transfers among participants, ensuring transparency and efficiency.
* **Currency Support**: Handle different currencies with specific precision.
* **Localization**: Support for multiple languages, including English (`en-US`) and Italian (`it-IT`).
* **Command-Based Interaction**: Use commands to interact with the bot, such as adding travelers, setting currencies, or viewing balances.
* **Database Integration**: Persistent storage of data using a database backend.
* **Logging**: Detailed logging for debugging and tracking application behavior.

## Use Cases and Examples

Travel-RS Bot was created to avoid the use of third-party applications for managing group expenses. Here are some examples of how it can be used:

* **Trip with Friends**: A group of friends goes on vacation together. They use the bot to track shared expenses like accommodation, meals, and activities.
* **Business Trip**: Colleagues traveling for a project can use the bot to split reimbursable travel expenses.
* **School Trip**: Students and teachers can manage expenses during a school trip, ensuring everyone pays their fair share.
* **Event Planning**: Event organizers can use the bot to manage shared costs among participants.

## Available Commands

The following commands are supported by Travel-RS Bot:

* **`/help`** — Displays a help message for the specified command. If no command is specified, it shows descriptions for all commands.

    * Example: `/help addexpense`
* **`/setlanguage`** — Sets the default language for the bot in the chat.

    * Example: `/setlanguage it-IT`
* **`/setcurrency`** — Sets the default currency for the travel plan.

    * Example: `/setcurrency EUR`
* **`/addtraveler`** — Adds a traveler with the specified name to the travel plan.

    * Example: `/addtraveler Alice`
* **`/deletetraveler`** — Removes the traveler with the specified name from the travel plan.

    * Example: `/deletetraveler Alice`
* **`/listtravelers`** — Displays the travelers in the travel plan.

    * Example: `/listtravelers`
* **`/addexpense`** — Starts a new interactive session to add an expense to the travel plan.

    * Example: `/addexpense` (a series of interactive questions will follow)
* **`/deleteexpense`** — Deletes the expense with the specified identifier from the travel plan.

    * Example: `/deleteexpense 3`
* **`/listexpenses`** — Displays the expenses in the travel plan. If a description is specified, it shows only the expenses matching the provided description. Supports fuzzy search for flexible matching.

    * Example: `/listexpenses`
    * Example: `/listexpenses Toll`
* **`/showexpense`** — Displays the details of the expense with the specified identifier.

    * Example: `/showexpense 3`
* **`/transfer`** — Transfers the specified amount from one traveler to another.

    * Example: `/transfer Alice Bob 25.00`
* **`/deletetransfer`** — Deletes the transfer with the specified identifier from the travel plan.

    * Example: `/deletetransfer 7`
* **`/listtransfers`** — Displays the transfers in the travel plan. If a name is specified, it shows only the transfers involving that traveler.

    * Example: `/listtransfers`
    * Example: `/listtransfers Alice`
* **`/showbalances`** — Displays simplified balances for all travelers, minimizing the total number of transfers needed to settle debts. If a name is specified, it shows the balance for the specified traveler.

    * Example: `/showbalances`
    * Example: `/showbalances Alice`
* **`/showstats`** — Displays statistics for expenses, transfers and travelers.

    * Example: `/showstats`
* **`/cancel`** — Cancels the currently running interactive dialogue.

    * Example: `/cancel`

These commands provide a comprehensive way to interact with the bot and effectively manage travel-related expenses.

## Detailed Usage Examples

Here are some detailed examples of how to use the bot to manage expenses for a group trip:

1.  **Adding Travelers**:

    A group of friends, Alice, Bob, and Charles, are organizing a trip. Add them using the `/addtraveler` command:

    ```
    User: /addtraveler Alice
    Bot:  Traveler Alice added successfully.
    
    User: /addtraveler Bob
    Bot:  Traveler Bob added successfully.
    
    User: /addtraveler Charles
    Bot:  Traveler Charles added successfully.
    ```

2.  **Viewing Travelers**:

    To confirm that everyone has been added correctly, use the `/listtravelers` command:

    ```
    User: /listtravelers
    Bot:  Alice
          Bob
          Charles
    ```

3.  **Adding an Expense**:

    Alice pays $50 for the highway toll. To record the expense, use the `/addexpense` command to start a conversation with the bot:

    ```
    User: /addexpense
    Bot:  The process can be interrupted at any time by sending `/cancel`.

          Set a description for this expense.
    User: Highway toll
    Bot:  How much is the expense?
    User: 50
    Bot:  Who paid for this?
    User: Alice
    Bot:  How would you like to split the expense? Type `/help addexpense` for more info.
    User: all
    Bot:  Expense recorded successfully!

          Expense #1: Highway toll - $50
    ```

    This will record an expense of $50 paid by Alice, divided equally among Alice, Bob, and Charles.

4.  **Cancelling a Dialogue**:

    To interrupt an ongoing dialogue, use the `/cancel` command:

    ```
    User: /cancel
    Bot:  The process was cancelled.
    ```

5.  **Viewing Expenses**:

    To see all the expenses recorded so far, use the `/listexpenses` command:

    ```
    User: /listexpenses
    Bot:  Expense #1: Highway toll - $50
          Expense #2: Hotel - $150
    ```

    You can filter expenses by specifying a search term:

    ```
    User: /listexpenses Toll
    Bot:  Expense #1: Highway toll - $50
    ```

    The bot will display a list of expenses, including the expense numeric IDs.

6.  **Deleting an Expense**:

    If there is an error in entering an expense, it can be deleted using `/deleteexpense` followed by the expense ID:

    ```
    User: /deleteexpense 2
    Bot:  Expense #2 deleted successfully.
    ```

7.  **Viewing Expense Details**:

    The `/listexpenses` command provides a concise summary of expenses, displaying their numeric IDs, descriptions, and total amounts. To view detailed information about a specific expense, use the `/showexpense` command followed by the expense ID:

    ```
    User: /showexpense 1
    Bot:  Number: 1 - Description: Highway Toll
          Amount: $50
          Paid by: Alice
          Shares:
          - Alice: $16.67
          - Bob: $16.67
          - Charles: $16.67
    ```

    This command reveals comprehensive details about the selected expense, including:
    - The numeric identifier
    - The description
    - The total amount spent
    - The name of the payer
    - A breakdown of the shares for each participant

8.  **Viewing Balances**:

    To see how much everyone owes or is owed, use the `/showbalances` command:

    ```
    User: /showbalances
    Bot:  Bob owes $16.67 to Alice.
          Charles owes $16.67 to Alice.
    ```

    To see the balance of a single traveler:

    ```
    User: /showbalances Bob
    Bot:  Bob owes $16.67 to Alice.
    ```

    The bot will display the simplified balances.

9.  **Recording a Transfer**:

    Bob pays $16.67 to Alice for his share of the toll. To record the transfer, use `/transfer` followed by the sender name, the receiver name, and the transferred amount:

    ```
    User: /transfer Bob Alice 16.67
    Bot:  Transfer recorded successfully.
    ```

10. **Viewing Transfers**:

    To see all the transfers made, use the `/listtransfers` command:

    ```
    User: /listtransfers
    Bot:  Transfer #1: Bob sent $16.67 to Alice
    ```

    To see transfers to or from a specific traveler:

    ```
    User: /listtransfers Alice
    Bot:  Transfer #1: Bob sent $16.67 to Alice
    ```

## Configuration

Travel-RS Bot is highly configurable, allowing users to tailor its behavior to their needs. Configuration is managed through TOML files.

### Main Configuration File

The primary configuration file is `config/config.toml`. This file specifies the active `profile` to be used by the bot. All other settings, such as database connection details, logging preferences, and the default language, are retrieved from the specified profile file located in the `config/profiles/` directory.

### Profile-Specific Configurations

Profile-specific configurations can be added in the `config/profiles/` directory. These profiles allow users to define settings for different environments or use cases. For example, a `dev-local.toml` profile might be used for local development, while a `production.toml` profile could be used for deployment.

Users can select a profile at runtime by using the `--profile` command-line option. When this option is specified, the bot will load its settings from the designated profile instead of the default profile defined in `config.toml`.

#### Profile-Specific Configuration Structure

The profile-specific configuration files are structured into sections, each serving a specific purpose. Below is a description of the key sections:

* `[logging]`

    * **`path`**: Specifies the directory where log files are stored.
    * **`file_name_prefix`**: Defines the prefix for log file names. The log files will be named by appending the date in UTC format to the prefix.
    * **`level`**: Sets the log level. Supported values are `"error"`, `"warn"`, `"info"`, `"debug"`, and `"trace"` or their corresponding numeric values (from `1` to `5` respectively).

* `[bot]`

    * **`token_source`**: Determines how the bot token is provided. Possible values:

        * `"file"`: Reads the token from a file.
        * `"env"`: Reads the token from an environment variable.
        * `"string"`: Uses the token directly as a string.
    * **`token`**: Specifies the bot token. Its value depends on `token_source`:

        * If `"file"`, this is the path to the file containing the token.
        * If `"env"`, this is the name of the environment variable holding the token.
        * If `"string"`, this is the token itself.

* `[database]`

    * **`address`**: The address of the database server (e.g., `ws://127.0.0.1:8000`).
    * **`username`**: The username for database authentication.
    * **`password`**: The password for database authentication.
    * **`namespace`**: The namespace used in the database.
    * **`database`**: The name of the database.

* `[i18n]`

    * **`default_locale`**: Specifies the default locale for the bot (e.g., `"en-US"`).
    * **`locales_path`**: Path to the directory containing localization files.
    * **`default_currency`**: Sets the default currency for formatting purposes (e.g., `"USD"`).

This modular structure allows users to easily configure the bot's behavior for different environments or use cases.

### Logging Configuration

Logs are written to files within a **profile-specific subfolder** located under the **path** specified in the `[logging]` section of your profile's `.toml` file. For instance, if the specified profile is `dev` and the `path` in `dev.toml` is set to `logs`, you'll find your logs in `./logs/dev/`. Each log file is timestamped for easy reference. You can customize logging behavior, including the log directory and log level, directly within your profile-specific configuration.

## Localization

### Customizing Fluent Localization Files

Travel-RS Bot uses [Fluent](https://projectfluent.org/) localization files to support multiple languages, making it easy to customize and extend language support. Localization files are organized into folders named after the locale code (e.g., `en-US`, `it-IT`) in the `locales/` directory. Each locale folder contains `.ftl` files and may include subfolders for further organization.

#### Available Fluent Localization Files

Each locale directory contains the following `.ftl` files, each serving a specific purpose for localization:

* `commands.ftl`: Includes translations for bot command responses, ensuring users can interact with the bot in their preferred language.
* `errors.ftl`: Provides localized error messages to help users understand issues in their language.
* `format.ftl`: Contains translations for formatting-related strings of custom type formats, such as details for expenses, shares, or transfers.
* `help.ftl`: Includes descriptions, help text, and usage instructions for various bot commands.
* The `dialogues/` folder contains translation files for interactive dialogues:

    * `dialogues/add-expense.ftl`: Handles translations for dialogues related to adding expenses, including prompts and confirmations.

These files are organized by locale (e.g., `en-US/messages.ftl`, `it-IT/messages.ftl`) to ensure seamless language support and easy customization.

#### Adding or Modifying Translations

1.  **Locate the Locale Folder**: Navigate to the folder corresponding to the desired locale in the `locales/` directory.
2.  **Edit or Add Messages**: Open the appropriate `.ftl` file and use Fluent's syntax to define or update translations. For example:

    ```
    welcome-message = Welcome to Travel-RS Bot!
    ```

3.  **Save Changes**: Save the file. A restart of the bot is necessary for the changes to take effect.

#### Adding a New Language

1.  **Create a New Locale Folder**: Add a new folder in the `locales/` directory with the appropriate locale code (e.g., `fr-FR` for French).
2.  **Add .ftl Files**: Populate the folder with `.ftl` files containing translations for all required messages.
3.  **Update Configuration**: Ensure the new language is listed in the bot's configuration or accessible via the `/setlanguage` command.

This structure ensures that localization is both flexible and scalable, allowing contributors to easily adapt the bot for different languages and regions. Users can switch languages using the `/setlanguage` command.

## Database Setup

See the [database README](database/README.md) file.

## Docker Setup

Travel-RS Bot can be run as a Docker container, with the container published to GitHub Container Registry with each release.

### Running with Docker Compose (Recommended)

Using Docker Compose simplifies managing the Travel-RS Bot container and its volumes. A [`docker-compose.yml`](docker-compose.yml) file is provided in the same directory as this `README`.

1.  **Ensure prerequisites:**
    * **Docker Desktop** (Windows/macOS) or **Docker Engine** (Linux) is installed and running.
    * You'll need to prepare the local directories for configuration, locales, and logs. By default, the `docker-compose.yml` expects these to be named `config`, `locales`, and `logs` in the **same directory as your `docker-compose.yml` file**. These directories will be mounted as volumes into the container. **You can, however, modify the volume paths in the `docker-compose.yml` if you prefer to store them elsewhere.**
    
        ```yaml
        # docker-compose.yml volumes configuration:
        volumes:
        - /path/to/config:/app/config
        - /path/to/locales:/app/locales
        - /path/to/logs:/app/logs
        ```

2.  **Prepare your configuration:**
    * Place your `config.toml` file inside your chosen config volume location (`./config` by default).
    * Create a `profiles/` directory inside your chosen config volume location (`./config` by default). and add your profile-specific configurations there.
    * Ensure your locale files are placed in your chosen locales volume location (`./locales` by default), organized by locale (e.g., `./locales/en-US`, `./locales/it-IT`).

3.  **Start the container:**
    Navigate to the directory containing `docker-compose.yml` in your terminal or PowerShell, then run:

    ```bash
    docker-compose up -d
    ```
    This command will:
    * Pull the `ghcr.io/princeofborgo/travel-rs:latest` image if it's not already present.
    * Create and start a container named `travel-rs`.
    * Mount the local directories (as defined in your `docker-compose.yml`) to `/app/config`, `/app/locales`, and `/app/logs` inside the container, respectively.
    * Configure the container to restart automatically unless explicitly stopped (`restart: unless-stopped`).

4.  **Update to the Latest Image:**
    If you are already running the bot and want to ensure you have the absolute latest version of the `ghcr.io/princeofborgo/travel-rs:latest` image, use the following commands:

    ```bash
    docker-compose pull travel-rs # Pulls the latest image for the 'travel-rs' service
    docker-compose up -d          # Recreates the container using the newly pulled image
    ```
    The `docker-compose pull` command explicitly downloads the freshest image from the registry. Then, `docker-compose up -d` will detect that the image has changed and recreate the `travel-rs` container with the new image, while preserving your data volumes.

5.  **Stop the container:**
    To stop and remove the container, run:

    ```bash
    docker-compose down
    ```

6.  **View logs:**
    To view the logs of the running container:

    ```bash
    docker-compose logs -f travel-rs
    ```

### Running from GitHub Container Registry (Manual Docker Commands)

While Docker Compose is recommended for its simplicity, you can also run the container directly using `docker run` commands.

First, pull the latest image:

```powershell
docker pull ghcr.io/princeofborgo/travel-rs:latest
```

Then, run the container, ensuring you map your local configuration, locales, and logs directories to the container's expected paths.

```powershell
# Run the container (PowerShell)
docker run -d `
    -v "C:/path/to/config:/app/config" `
    -v "C:/path/to/locales:/app/locales" `
    -v "C:/path/to/logs:/app/logs" `
    --name travel-rs `
    ghcr.io/princeofborgo/travel-rs:latest

# Example using the current directory (PowerShell)
docker run -d `
    -v "$PWD/config:/app/config" `
    -v "$PWD/locales:/app/locales" `
    -v "$PWD/logs:/app/logs" `
    --name travel-rs `
    ghcr.io/princeofborgo/travel-rs:latest
```

For Linux/Unix systems:
```bash
# Pull the latest image
docker pull ghcr.io/princeofborgo/travel-rs:latest

# Run the container (Linux/Unix)
docker run -d \
    -v "/path/to/config:/app/config" \
    -v "/path/to/locales:/app/locales" \
    -v "/path/to/logs:/app/logs" \
    --name travel-rs \
    ghcr.io/princeofborgo/travel-rs:latest

# Example using the current directory (Linux/Unix)
docker run -d \
    -v "$(pwd)/config:/app/config" \
    -v "$(pwd)/locales:/app/locales" \
    -v "$(pwd)/logs:/app/logs" \
    --name travel-rs \
    ghcr.io/princeofborgo/travel-rs:latest
```

### Configuration

The container expects configuration files to be mounted at `/app/config`. Make sure your local `config` directory contains:
- `config.toml`: Main configuration file
- `profiles/`: Directory containing profile-specific configurations

### Volume Configuration

The container requires three volume mounts to function properly:

1. **`config` Volume** (mounted at `/app/config`)
   - Contains essential configuration files
   - Required for bot token and profile settings
   - Must include:
     - `config.toml`: Main configuration file
     - `profiles/`: Directory with environment-specific settings

2. **`locales` Volume** (mounted at `/app/locales`)
   - Contains Fluent translation files
   - Required for multi-language support
   - Organized by locale (e.g., `en-US`, `it-IT`)

3. **`logs` Volume** (mounted at `/app/logs`)
   - Persists application logs across container restarts
   - Enables log access from the host machine
   - Files are organized by profile and date

## Roadmap

Planned features and improvements are documented in the [ROADMAP.md](ROADMAP.md) file.

## Changelog

For a history of changes and updates, see the [CHANGELOG.md](CHANGELOG.md) file.

## Contact

For questions or support, please contact the project maintainers or open an issue on GitHub.

© 2025 Michele Medori. All rights reserved.

[github badge]: https://img.shields.io/badge/github-PrinceOfBorgo%2Ftravel--rs-8da0cb?logo=github&logoColor=white
[github link]: https://github.com/PrinceOfBorgo/travel-rs

[build & test badge]: https://img.shields.io/github/actions/workflow/status/PrinceOfBorgo/travel-rs/ci.yml?logo=githubactions&logoColor=white&label=Cargo%20Build%20%26%20Test
[build & test link]: https://github.com/PrinceOfBorgo/travel-rs/actions/workflows/ci.yml

[release badge]: https://img.shields.io/github/actions/workflow/status/PrinceOfBorgo/travel-rs/release.yml?logo=githubactions&logoColor=white&label=Release
[release link]: https://github.com/PrinceOfBorgo/travel-rs/actions/workflows/release.yml

[codecov badge]: https://img.shields.io/codecov/c/github/PrinceOfBorgo/travel-rs?logo=codecov&logoColor=white
[codecov link]: https://codecov.io/gh/PrinceOfBorgo/travel-rs
