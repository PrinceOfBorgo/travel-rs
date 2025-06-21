pub mod args;
pub mod commands;
pub mod dialogues;
pub mod errors;
pub mod format;
pub mod help;
pub mod terms;
mod to_fluent_datetime;
mod translate;

use indoc::formatdoc;
pub use to_fluent_datetime::ToFluentDateTime;
pub use translate::{Translate, TranslateWithArgs};

use crate::{commands::Command, consts::*, settings::SETTINGS};
use commands::COMMAND_DESCRIPTIONS;
use fluent::FluentResource;
use fluent_datetime::BundleExt;
use fluent_templates::{ArcLoader, Loader};
use std::sync::{Arc, LazyLock};
use teloxide::utils::command::BotCommands;
use terms::*;
use unic_langid::LanguageIdentifier;

static LOCALES: LazyLock<ArcLoader> = LazyLock::new(|| {
    let location = &SETTINGS.i18n.locales_path;
    let fallback = SETTINGS.i18n.default_locale.clone();

    ArcLoader::builder(location, fallback)
        .customize(|bundle| {
            let commands = formatdoc!(
                "
                {HELP_COMMAND} = {help}
                {SET_LANGUAGE_COMMAND} = {set_language}
                {SET_CURRENCY_COMMAND} = {set_currency}
                {ADD_TRAVELER_COMMAND} = {add_traveler}
                {DELETE_TRAVELER_COMMAND} = {delete_traveler}
                {LIST_TRAVELERS_COMMAND} = {list_travelers}
                {ADD_EXPENSE_COMMAND} = {add_expense}
                {DELETE_EXPENSE_COMMAND} = {delete_expense}
                {LIST_EXPENSES_COMMAND} = {list_expenses}
                {SHOW_EXPENSE_COMMAND} = {show_expense}
                {TRANSFER_COMMAND} = {transfer}
                {DELETE_TRANSFER_COMMAND} = {delete_transfer}
                {LIST_TRANSFERS_COMMAND} = {list_transfers}
                {SHOW_BALANCES_COMMAND} = {show_balances}
                {SHOW_STATS_COMMAND} = {show_stats}
                {CANCEL_COMMAND} = {cancel}
                ",
                help = variant_to_string!(Command::Help),
                set_language = variant_to_string!(Command::SetLanguage),
                set_currency = variant_to_string!(Command::SetCurrency),
                add_traveler = variant_to_string!(Command::AddTraveler),
                delete_traveler = variant_to_string!(Command::DeleteTraveler),
                list_travelers = variant_to_string!(Command::ListTravelers),
                add_expense = variant_to_string!(Command::AddExpense),
                delete_expense = variant_to_string!(Command::DeleteExpense),
                list_expenses = variant_to_string!(Command::ListExpenses),
                show_expense = variant_to_string!(Command::ShowExpense),
                transfer = variant_to_string!(Command::Transfer),
                delete_transfer = variant_to_string!(Command::DeleteTransfer),
                list_transfers = variant_to_string!(Command::ListTransfers),
                show_balances = variant_to_string!(Command::ShowBalances),
                show_stats = variant_to_string!(Command::ShowStats),
                cancel = variant_to_string!(Command::Cancel),
            );

            let consts = formatdoc!(
                "
                {I18N_DECIMAL_SEP} = {decimal_sep}
                {I18N_SPLIT_AMONG_ENTRIES_SEP} = {split_among_entries_sep}
                {I18N_SPLIT_AMONG_NAME_AMOUNT_SEP} = {split_among_name_amount_sep}
                {I18N_ALL_KWORD} = {all_kword}
                {I18N_END_KWORD} = {end_kword}
                ",
                decimal_sep = DECIMAL_SEP,
                split_among_entries_sep = SPLIT_AMONG_ENTRIES_SEP,
                split_among_name_amount_sep = SPLIT_AMONG_NAME_AMOUNT_SEP,
                all_kword = ALL_KWORD,
                end_kword = END_KWORD
            );

            let command_descriptions = formatdoc!(
                "
                {COMMAND_DESCRIPTIONS} = {descriptions}
                ",
                descriptions = Command::descriptions()
                    .to_string()
                    .lines()
                    .collect::<Vec<&str>>()
                    .join("\n    ") // Indent each line with 4 spaces for Fluent multiline strings syntax
            );

            bundle
                .add_resource(Arc::new(
                    FluentResource::try_new(commands)
                        .expect("Failed to create FluentResource from commands"),
                ))
                .expect("Failed to add resource to bundle");

            bundle
                .add_resource(Arc::new(
                    FluentResource::try_new(consts)
                        .expect("Failed to create FluentResource from consts"),
                ))
                .expect("Failed to add resource to bundle");

            bundle
                .add_resource(Arc::new(
                    FluentResource::try_new(command_descriptions)
                        .expect("Failed to create FluentResource from command_descriptions"),
                ))
                .expect("Failed to add resource to bundle");

            bundle.set_use_isolating(false);

            // Register the DATETIME function
            bundle
                .add_datetime_support()
                .expect("DATETIME function should be supported");
        })
        .build()
        .expect("Failed to build ArcLoader")
});

pub fn is_lang_available(langid: &LanguageIdentifier) -> bool {
    LOCALES.locales().any(|locale| locale == langid)
}

pub fn available_langs() -> Box<dyn Iterator<Item = LanguageIdentifier>> {
    Box::new(LOCALES.locales().cloned())
}
