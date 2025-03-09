pub mod args;
pub mod commands;
pub mod dialogues;
pub mod errors;
pub mod help;
pub mod terms;
mod translatable;
pub mod types;

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
pub use translatable::Translatable;

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
{FIND_EXPENSES_COMMAND} = {find_expenses}
{HELP_COMMAND} = {help}
{LIST_EXPENSES_COMMAND} = {list_expenses}
{LIST_TRAVELERS_COMMAND} = {list_travelers}
{SHOW_BALANCE_COMMAND} = {show_balance}
{SHOW_BALANCES_COMMAND} = {show_balances}
{SHOW_EXPENSE_COMMAND} = {show_expense}
{TRANSFER_COMMAND} = {transfer}
",
                cancel = variant_to_string!(Command::Cancel),
                add_expense = variant_to_string!(Command::AddExpense),
                add_traveler = variant_to_string!(Command::AddTraveler),
                delete_expense = variant_to_string!(Command::DeleteExpense),
                delete_traveler = variant_to_string!(Command::DeleteTraveler),
                find_expenses = variant_to_string!(Command::FindExpenses),
                help = variant_to_string!(Command::Help),
                list_expenses = variant_to_string!(Command::ListExpenses),
                list_travelers = variant_to_string!(Command::ListTravelers),
                show_balance = variant_to_string!(Command::ShowBalance),
                show_balances = variant_to_string!(Command::ShowBalances),
                show_expense = variant_to_string!(Command::ShowExpense),
                transfer = variant_to_string!(Command::Transfer)
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
    let langid = &ctx.lock().expect("Failed to lock context").langid;
    LOCALES
        .try_lookup(langid, input)
        .unwrap_or(input.to_owned())
}

pub fn translate_with_args(
    ctx: Arc<Mutex<Context>>,
    input: &str,
    args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
) -> String {
    let langid = &ctx.lock().expect("Failed to lock context").langid;
    LOCALES
        .try_lookup_with_args(langid, input, args)
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
