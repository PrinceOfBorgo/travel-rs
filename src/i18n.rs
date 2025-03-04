use fluent::{FluentResource, FluentValue};
use fluent_templates::Loader;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::RandomState;
use std::sync::{Arc, LazyLock, Mutex};

use crate::Context;

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
        customise: |bundle| {
            static RESOURCES: LazyLock<FluentResource> = LazyLock::new(|| {
                let resources = format!("
-i18n-cancel-command = {cancel}
-i18n-addexpense-command = {add_expense}
-i18n-addtraveler-command = {add_traveler}
", 
cancel = variant_to_string!(Command::Cancel),
add_expense = variant_to_string!(Command::AddExpense),
add_traveler = variant_to_string!(Command::AddTraveler)
);

                FluentResource::try_new(resources).unwrap()
            });

            bundle.add_resource(&RESOURCES).expect("Failed to add FTL resources to the bundle.");
        }
    };
}

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
