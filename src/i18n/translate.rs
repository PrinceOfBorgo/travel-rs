use fluent::FluentValue;
use fluent_templates::Loader;

use crate::{Context, i18n::LOCALES};
use std::{
    borrow::Cow,
    collections::HashMap,
    hash::RandomState,
    sync::{Arc, Mutex},
};

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
            .unwrap_or(self.as_ref().to_owned())
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
            .unwrap_or(self.as_ref().to_owned())
    }
}
