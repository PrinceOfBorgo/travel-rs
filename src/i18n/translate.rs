use fluent::FluentValue;
use fluent_templates::Loader;

use crate::{Context, i18n::LOCALES};
use std::{
    borrow::Cow,
    collections::HashMap,
    hash::RandomState,
    sync::{Arc, Mutex},
};

/// Strict Fluent lookup: returns the localized string when the key is
/// defined for the active locale (or its in-language fallback chain),
/// or `None` when it is not.
///
/// Unlike [`Translate`], this trait does **not** consult the configured
/// default fallback locale (e.g. `en-US`): if the key is defined only
/// there, the lookup returns `None`. This is the right behavior when the
/// caller wants to detect a genuinely missing translation and provide a
/// custom fallback.
pub trait TryTranslate {
    fn try_translate_with_indent(
        &self,
        ctx: Arc<Mutex<Context>>,
        indent_lvl: usize,
    ) -> Option<String>;

    fn try_translate_with_indent_default(&self, indent_lvl: usize) -> Option<String> {
        self.try_translate_with_indent(Arc::new(Mutex::new(Context::default())), indent_lvl)
    }

    fn try_translate(&self, ctx: Arc<Mutex<Context>>) -> Option<String> {
        self.try_translate_with_indent(ctx, 0)
    }

    fn try_translate_default(&self) -> Option<String> {
        self.try_translate_with_indent_default(0)
    }
}

pub trait TryTranslateWithArgs {
    fn try_translate_with_args_indent(
        &self,
        ctx: Arc<Mutex<Context>>,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
        indent_lvl: usize,
    ) -> Option<String>;

    fn try_translate_with_args_indent_default(
        &self,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
        indent_lvl: usize,
    ) -> Option<String> {
        self.try_translate_with_args_indent(
            Arc::new(Mutex::new(Context::default())),
            args,
            indent_lvl,
        )
    }

    fn try_translate_with_args(
        &self,
        ctx: Arc<Mutex<Context>>,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
    ) -> Option<String> {
        self.try_translate_with_args_indent(ctx, args, 0)
    }

    fn try_translate_with_args_default(
        &self,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
    ) -> Option<String> {
        self.try_translate_with_args_indent_default(args, 0)
    }
}

pub trait Translate {
    fn translate_with_indent(&self, ctx: Arc<Mutex<Context>>, indent_lvl: usize) -> String;

    fn translate_with_indent_default(&self, indent_lvl: usize) -> String {
        self.translate_with_indent(Arc::new(Mutex::new(Context::default())), indent_lvl)
    }

    fn translate(&self, ctx: Arc<Mutex<Context>>) -> String {
        self.translate_with_indent(ctx, 0)
    }

    fn translate_default(&self) -> String {
        self.translate_with_indent_default(0)
    }
}

pub trait TranslateWithArgs {
    fn translate_with_args_indent(
        &self,
        ctx: Arc<Mutex<Context>>,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
        indent_lvl: usize,
    ) -> String;

    fn translate_with_args_indent_default(
        &self,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
        indent_lvl: usize,
    ) -> String {
        self.translate_with_args_indent(Arc::new(Mutex::new(Context::default())), args, indent_lvl)
    }

    fn translate_with_args(
        &self,
        ctx: Arc<Mutex<Context>>,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
    ) -> String {
        self.translate_with_args_indent(ctx, args, 0)
    }

    fn translate_with_args_default(
        &self,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
    ) -> String {
        self.translate_with_args_indent_default(args, 0)
    }
}

impl<T: AsRef<str>> TryTranslate for T {
    fn try_translate_with_indent(
        &self,
        ctx: Arc<Mutex<Context>>,
        indent_lvl: usize,
    ) -> Option<String> {
        let langid = {
            let ctx = ctx.lock().expect("Failed to lock context");
            ctx.langid.clone()
        };
        let indentation = " ".repeat(indent_lvl * 4); // 4 spaces per indent level
        LOCALES
            .lookup_no_default_fallback(
                &langid,
                self.as_ref(),
                None::<&HashMap<Cow<'static, str>, FluentValue<'_>>>,
            )
            .map(|s| format!("{indentation}{s}"))
    }
}

impl<T: AsRef<str>> TryTranslateWithArgs for T {
    fn try_translate_with_args_indent(
        &self,
        ctx: Arc<Mutex<Context>>,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
        indent_lvl: usize,
    ) -> Option<String> {
        let langid = {
            let ctx = ctx.lock().expect("Failed to lock context");
            ctx.langid.clone()
        };
        let indentation = " ".repeat(indent_lvl * 4); // 4 spaces per indent level
        LOCALES
            .lookup_no_default_fallback(&langid, self.as_ref(), Some(args))
            .map(|s| format!("{indentation}{s}"))
    }
}

impl<T: AsRef<str>> Translate for T {
    fn translate_with_indent(&self, ctx: Arc<Mutex<Context>>, indent_lvl: usize) -> String {
        let langid = {
            let ctx = ctx.lock().expect("Failed to lock context");
            ctx.langid.clone()
        };
        let indentation = " ".repeat(indent_lvl * 4); // 4 spaces per indent level
        LOCALES
            .try_lookup(&langid, self.as_ref())
            .map(|s| format!("{indentation}{s}"))
            .unwrap_or_else(|| self.as_ref().to_owned())
    }
}

impl<T: AsRef<str>> TranslateWithArgs for T {
    fn translate_with_args_indent(
        &self,
        ctx: Arc<Mutex<Context>>,
        args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
        indent_lvl: usize,
    ) -> String {
        let langid = {
            let ctx = ctx.lock().expect("Failed to lock context");
            ctx.langid.clone()
        };
        let indentation = " ".repeat(indent_lvl * 4); // 4 spaces per indent level
        LOCALES
            .try_lookup_with_args(&langid, self.as_ref(), args)
            .map(|s| format!("{indentation}{s}"))
            .unwrap_or_else(|| self.as_ref().to_owned())
    }
}
