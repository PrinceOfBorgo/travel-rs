pub mod args;
pub mod commands;
pub mod dialogues;
pub mod errors;
pub mod help;
pub mod terms;
mod translate;
pub mod types;

pub use translate::Translate;

use crate::{Context, commands::Command, consts::*, settings::SETTINGS};
use commands::COMMAND_DESCRIPTIONS;
use fluent::{FluentResource, FluentValue};
use fluent_templates::{ArcLoader, Loader};
use std::{
    borrow::Cow,
    collections::HashMap,
    hash::RandomState,
    sync::{Arc, LazyLock, Mutex},
};
use teloxide::utils::command::BotCommands;
use terms::*;
use unic_langid::LanguageIdentifier;

static LOCALES: LazyLock<ArcLoader> = LazyLock::new(|| {
    let location = &SETTINGS.i18n.locales_path;
    let fallback = SETTINGS.i18n.default_locale.clone();

    ArcLoader::builder(location, fallback)
        .customize(|bundle| {
            let commands = format!(
                "
{CANCEL_COMMAND} = {cancel}
{ADD_EXPENSE_COMMAND} = {add_expense}
{ADD_TRAVELER_COMMAND} = {add_traveler}
{DELETE_EXPENSE_COMMAND} = {delete_expense}
{DELETE_TRAVELER_COMMAND} = {delete_traveler}
{HELP_COMMAND} = {help}
{LIST_EXPENSES_COMMAND} = {list_expenses}
{LIST_TRAVELERS_COMMAND} = {list_travelers}
{SET_CURRENCY_COMMAND} = {set_currency}
{SET_LANGUAGE_COMMAND} = {set_language}
{SHOW_BALANCES_COMMAND} = {show_balances}
{SHOW_EXPENSE_COMMAND} = {show_expense}
{TRANSFER_COMMAND} = {transfer}
{DELETE_TRANSFER_COMMAND} = {delete_transfer}
{LIST_TRANSFERS_COMMAND} = {list_transfers}
",
                cancel = variant_to_string!(Command::Cancel),
                add_expense = variant_to_string!(Command::AddExpense),
                add_traveler = variant_to_string!(Command::AddTraveler),
                delete_expense = variant_to_string!(Command::DeleteExpense),
                delete_traveler = variant_to_string!(Command::DeleteTraveler),
                help = variant_to_string!(Command::Help),
                list_expenses = variant_to_string!(Command::ListExpenses),
                list_travelers = variant_to_string!(Command::ListTravelers),
                set_currency = variant_to_string!(Command::SetCurrency),
                set_language = variant_to_string!(Command::SetLanguage),
                show_balances = variant_to_string!(Command::ShowBalances),
                show_expense = variant_to_string!(Command::ShowExpense),
                transfer = variant_to_string!(Command::Transfer),
                delete_transfer = variant_to_string!(Command::DeleteTransfer),
                list_transfers = variant_to_string!(Command::ListTransfers),
            );

            let consts = format!(
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

            let command_descriptions = format!(
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

            bundle.set_use_isolating(false)
        })
        .build()
        .expect("Failed to build ArcLoader")
});

pub fn translate(ctx: Arc<Mutex<Context>>, input: &str) -> String {
    let langid = {
        let ctx = ctx.lock().expect("Failed to lock context");
        ctx.langid.clone()
    };
    LOCALES
        .try_lookup(&langid, input)
        .unwrap_or(input.to_owned())
}

pub fn translate_with_args(
    ctx: Arc<Mutex<Context>>,
    input: &str,
    args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
) -> String {
    let langid = {
        let ctx = ctx.lock().expect("Failed to lock context");
        ctx.langid.clone()
    };
    LOCALES
        .try_lookup_with_args(&langid, input, args)
        .unwrap_or(input.to_owned())
}

pub fn translate_default(input: &str) -> String {
    let langid = &SETTINGS.i18n.default_locale;
    LOCALES
        .try_lookup(langid, input)
        .unwrap_or(input.to_owned())
}

pub fn translate_with_args_default(
    input: &str,
    args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
) -> String {
    let langid = &SETTINGS.i18n.default_locale;
    LOCALES
        .try_lookup_with_args(langid, input, args)
        .unwrap_or(input.to_owned())
}

pub fn is_lang_available(langid: &LanguageIdentifier) -> bool {
    LOCALES.locales().any(|locale| locale == langid)
}

pub fn available_langs() -> Box<dyn Iterator<Item = LanguageIdentifier>> {
    Box::new(LOCALES.locales().cloned())
}
