//! Generic helpers shared across dialogue handlers.
//!
//! Every dialogue state implements [`DialogueState`], which exposes the
//! Fluent message id describing the running process. [`DialogueStorageDyn`]
//! is a small type-erased view over any
//! [`teloxide::dispatching::dialogue::Storage`] of such a state, used by
//! [`DialogueRegistry`] to treat all known dialogue storages uniformly
//! when checking for, labelling, or exiting an active dialogue.
//!
//! Adding a new dialogue requires (1) implementing [`DialogueState`] for
//! its state enum and (2) registering its storage in [`DialogueRegistry`]
//! (built in `deps()` in `main.rs`); nothing else in this module needs to
//! change.

use crate::{
    Context, HandlerResult,
    dialogues::{
        add_expense_dialogue::AddExpenseState,
        pending_command_dialogue::{PendingCommandState, PendingCommandStorage},
    },
    i18n::{self, Translate, TranslateWithArgs, args},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
};
use strum::{EnumIter, IntoEnumIterator};
use teloxide::{
    dispatching::dialogue::{InMemStorage, Storage},
    prelude::*,
};
use tracing::Level;

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Common interface every dialogue state implements. Centralises the
/// per-state behaviour (currently: producing the Fluent message id used
/// in the "another process is already running" notice) so the registry
/// machinery never needs to name concrete state types.
pub trait DialogueState: Default + Clone + Send + Sync + 'static {
    /// Fluent message id describing this dialogue to the user.
    fn running_label(&self) -> &'static str;
}

/// Type-erased view over a dialogue storage. Lets us hold heterogeneous
/// storages in a single collection.
pub trait DialogueStorageDyn: Send + Sync {
    /// `true` iff a dialogue exists in this storage for `chat_id`.
    fn is_running<'a>(&'a self, chat_id: ChatId) -> BoxFut<'a, bool>;
    /// Returns the Fluent message id describing the dialogue currently
    /// running for `chat_id` in this storage, or `None` if none is active.
    fn running_label<'a>(&'a self, chat_id: ChatId) -> BoxFut<'a, Option<&'static str>>;
    /// Drops the dialogue stored for `chat_id`, if any.
    fn try_exit<'a>(&'a self, chat_id: ChatId) -> BoxFut<'a, HandlerResult>;
}

/// Adapter that binds a concrete state type `D` to a [`Storage`] so it can
/// be type-erased behind [`DialogueStorageDyn`].
struct StorageAdapter<S, D>
where
    S: Storage<D> + ?Sized,
{
    storage: Arc<S>,
    _state: PhantomData<fn() -> D>,
}

impl<S, D> DialogueStorageDyn for StorageAdapter<S, D>
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: DialogueState,
{
    fn is_running<'a>(&'a self, chat_id: ChatId) -> BoxFut<'a, bool> {
        let storage = Arc::clone(&self.storage);
        Box::pin(async move { storage.get_dialogue(chat_id).await.ok().flatten().is_some() })
    }

    fn running_label<'a>(&'a self, chat_id: ChatId) -> BoxFut<'a, Option<&'static str>> {
        let storage = Arc::clone(&self.storage);
        Box::pin(async move {
            storage
                .get_dialogue(chat_id)
                .await
                .ok()
                .flatten()
                .map(|state| state.running_label())
        })
    }

    fn try_exit<'a>(&'a self, chat_id: ChatId) -> BoxFut<'a, HandlerResult> {
        let storage = Arc::clone(&self.storage);
        Box::pin(async move {
            if Arc::clone(&storage).get_dialogue(chat_id).await?.is_some() {
                Dialogue::new(storage, chat_id).exit().await?;
            }
            Ok(())
        })
    }
}

/// Wraps any concrete dialogue storage into a type-erased
/// [`DialogueStorageDyn`] suitable for [`DialogueRegistry`]. The label is
/// produced via the [`DialogueState`] impl on `D`.
pub fn erase<S, D>(storage: Arc<S>) -> Arc<dyn DialogueStorageDyn>
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: DialogueState,
{
    Arc::new(StorageAdapter::<S, D> {
        storage,
        _state: PhantomData,
    })
}

/// Type-erased collection of every known dialogue storage. Inject this as a
/// single dependency to operate on all dialogues without naming their
/// concrete state types.
#[derive(Clone)]
pub struct DialogueRegistry {
    storages: Arc<[Arc<dyn DialogueStorageDyn>]>,
}

/// Single source of truth for the set of dialogues the bot supports. Adding
/// a variant here is a compile error until [`DialogueRegistry::build`] is
/// updated to wire the new storage in (and, by chain, until the caller in
/// `main.rs` provides the storage), guaranteeing every dialogue participates
/// in cross-dialogue checks (`any_running`, `exit_all`).
#[derive(Debug, Clone, Copy, EnumIter)]
pub enum DialogueKind {
    AddExpense,
    PendingCommand,
}

/// Bundle of every concrete dialogue storage. Add a field when introducing
/// a new dialogue: the exhaustive `match` in [`DialogueRegistry::build`]
/// will then refuse to compile until you map the new variant to its field.
pub struct DialogueStorages {
    pub add_expense: Arc<InMemStorage<AddExpenseState>>,
    pub pending_command: Arc<PendingCommandStorage>,
}

impl DialogueRegistry {
    /// Build the registry from the bundle of concrete storages. The match
    /// inside is exhaustive over [`DialogueKind`], so the type system forces
    /// every variant to be wired up.
    pub fn build(storages: &DialogueStorages) -> Self {
        let entries: Vec<Arc<dyn DialogueStorageDyn>> = DialogueKind::iter()
            .map(|kind| match kind {
                DialogueKind::AddExpense => {
                    erase::<_, AddExpenseState>(Arc::clone(&storages.add_expense))
                }
                DialogueKind::PendingCommand => {
                    erase::<_, PendingCommandState>(Arc::clone(&storages.pending_command))
                }
            })
            .collect();
        Self {
            storages: entries.into(),
        }
    }

    /// `true` iff at least one registered storage has an active dialogue for
    /// `chat_id`.
    pub async fn any_running(&self, chat_id: ChatId) -> bool {
        for storage in self.storages.iter() {
            if storage.is_running(chat_id).await {
                return true;
            }
        }
        false
    }

    /// Returns the Fluent message id describing the first dialogue found to
    /// be running for `chat_id`, or `None` if no dialogue is active.
    pub async fn running_label(&self, chat_id: ChatId) -> Option<&'static str> {
        for storage in self.storages.iter() {
            if let Some(label) = storage.running_label(chat_id).await {
                return Some(label);
            }
        }
        None
    }

    /// Exits every active dialogue for `chat_id`. Returns `true` if at least
    /// one dialogue was actually present and exited.
    pub async fn exit_all(
        &self,
        chat_id: ChatId,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut any_exited = false;
        for storage in self.storages.iter() {
            if storage.is_running(chat_id).await {
                any_exited = true;
                storage.try_exit(chat_id).await?;
            }
        }
        Ok(any_exited)
    }
}

/// `dptree`-friendly filter: `true` iff *any* known dialogue is currently
/// active for the chat that sent `msg`.
pub async fn any_running(registry: DialogueRegistry, msg: Message) -> bool {
    registry.any_running(msg.chat.id).await
}

/// `dptree`-friendly filter: `true` iff the dialogue using
/// `InMemStorage<D>` is active for the chat that sent `msg`. Use turbofish
/// at the call site, e.g. `is_running::<AddExpenseState>`.
pub async fn is_running<D>(storage: Arc<InMemStorage<D>>, msg: Message) -> bool
where
    D: DialogueState,
{
    erase::<_, D>(storage).is_running(msg.chat.id).await
}

/// Endpoint used as a dispatcher leaf when a new dialogue would collide with
/// one that is already in progress. Sends the localized
/// `process-already-running` message (interpolating the name of the
/// dialogue that is currently running, when known) and stops dispatching.
#[apply(trace_skip_all)]
pub async fn process_already_running_endpoint(
    bot: Bot,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
    registry: DialogueRegistry,
) -> HandlerResult {
    let process_name = match registry.running_label(msg.chat.id).await {
        Some(label) => label.translate(Arc::clone(&ctx)),
        // Race: the offending dialogue exited between the filter and here.
        // Fall back to a generic placeholder so the message still renders.
        None => i18n::commands::RUNNING_PROCESS_UNKNOWN.translate(Arc::clone(&ctx)),
    };
    let text = i18n::commands::PROCESS_ALREADY_RUNNING.translate_with_args(
        ctx,
        &hashmap! { args::PROCESS.into() => process_name.into() },
    );
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogues::pending_command_dialogue::add_traveler::AddTravelerState;

    fn fresh_storages() -> DialogueStorages {
        DialogueStorages {
            add_expense: InMemStorage::<AddExpenseState>::new(),
            pending_command: PendingCommandStorage::new(),
        }
    }

    /// `DialogueKind::iter` enumerates every variant. If a new variant is
    /// added without updating this assertion, the test fails — a runtime
    /// reminder to revisit anything that depends on the variant set.
    #[test]
    fn dialogue_kind_iter_covers_every_variant() {
        let kinds: Vec<DialogueKind> = DialogueKind::iter().collect();
        assert_eq!(kinds.len(), 2);
        assert!(matches!(kinds[0], DialogueKind::AddExpense));
        assert!(matches!(kinds[1], DialogueKind::PendingCommand));
    }

    /// `build` produces one entry per [`DialogueKind`] variant.
    #[test]
    fn build_registers_every_kind() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        assert_eq!(registry.storages.len(), DialogueKind::iter().count());
    }

    #[tokio::test]
    async fn any_running_false_on_empty_registry() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        assert!(!registry.any_running(ChatId(1)).await);
    }

    #[tokio::test]
    async fn any_running_true_when_add_expense_active() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        let chat_id = ChatId(42);
        Arc::clone(&storages.add_expense)
            .update_dialogue(chat_id, AddExpenseState::default())
            .await
            .unwrap();
        assert!(registry.any_running(chat_id).await);
        // A different chat is unaffected.
        assert!(!registry.any_running(ChatId(43)).await);
    }

    #[tokio::test]
    async fn any_running_true_when_pending_command_active() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        let chat_id = ChatId(7);
        Arc::clone(&storages.pending_command)
            .update_dialogue(
                chat_id,
                PendingCommandState::AddTraveler(AddTravelerState::AskName),
            )
            .await
            .unwrap();
        assert!(registry.any_running(chat_id).await);
    }

    #[tokio::test]
    async fn exit_all_returns_false_when_nothing_running() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        let exited = registry.exit_all(ChatId(1)).await.unwrap();
        assert!(!exited);
    }

    /// `exit_all` clears every dialogue across every storage in one call.
    #[tokio::test]
    async fn exit_all_clears_every_active_dialogue() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        let chat_id = ChatId(99);

        // Seed both storages for the same chat.
        Arc::clone(&storages.add_expense)
            .update_dialogue(chat_id, AddExpenseState::default())
            .await
            .unwrap();
        Arc::clone(&storages.pending_command)
            .update_dialogue(
                chat_id,
                PendingCommandState::AddTraveler(AddTravelerState::AskName),
            )
            .await
            .unwrap();
        assert!(registry.any_running(chat_id).await);

        let exited = registry.exit_all(chat_id).await.unwrap();
        assert!(exited);
        assert!(!registry.any_running(chat_id).await);
        // Both concrete storages are now empty.
        assert!(
            Arc::clone(&storages.add_expense)
                .get_dialogue(chat_id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            Arc::clone(&storages.pending_command)
                .get_dialogue(chat_id)
                .await
                .unwrap()
                .is_none()
        );
    }

    /// `exit_all` is idempotent: a second call on a clean chat is a no-op.
    #[tokio::test]
    async fn exit_all_is_idempotent() {
        let storages = fresh_storages();
        let registry = DialogueRegistry::build(&storages);
        let chat_id = ChatId(5);
        Arc::clone(&storages.add_expense)
            .update_dialogue(chat_id, AddExpenseState::default())
            .await
            .unwrap();
        assert!(registry.exit_all(chat_id).await.unwrap());
        assert!(!registry.exit_all(chat_id).await.unwrap());
    }

    /// Erased storages preserve the per-state-type scoping: a dialogue
    /// active in one storage is not visible through another. This is the
    /// invariant `is_running::<D>` relies on.
    #[tokio::test]
    async fn erased_storage_is_scoped_to_state_type() {
        let storages = fresh_storages();
        let chat_id = ChatId(11);
        Arc::clone(&storages.add_expense)
            .update_dialogue(chat_id, AddExpenseState::default())
            .await
            .unwrap();

        let add_expense_erased = erase::<_, AddExpenseState>(Arc::clone(&storages.add_expense));
        let pending_erased = erase::<_, PendingCommandState>(Arc::clone(&storages.pending_command));

        assert!(add_expense_erased.is_running(chat_id).await);
        assert!(!pending_erased.is_running(chat_id).await);
    }
}
