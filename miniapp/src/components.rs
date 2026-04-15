use leptos::prelude::*;

use crate::types::{MutationResponse, TravelerResponse};

/// Signals for form feedback and submission state.
#[derive(Clone, Copy)]
pub struct FormState {
    pub feedback: ReadSignal<Option<String>>,
    pub set_feedback: WriteSignal<Option<String>>,
    pub is_err: ReadSignal<bool>,
    pub set_is_err: WriteSignal<bool>,
    pub submitting: ReadSignal<bool>,
    pub set_submitting: WriteSignal<bool>,
}

impl FormState {
    pub fn new() -> Self {
        let (feedback, set_feedback) = signal(Option::<String>::None);
        let (is_err, set_is_err) = signal(false);
        let (submitting, set_submitting) = signal(false);
        Self { feedback, set_feedback, is_err, set_is_err, submitting, set_submitting }
    }

    /// Handle a mutation result: set feedback + error flag. Returns `true` on success.
    pub fn handle(&self, result: Result<MutationResponse, String>, success_msg: String) -> bool {
        match result {
            Ok(r) if r.ok => {
                self.set_feedback.set(Some(success_msg));
                self.set_is_err.set(false);
                true
            }
            Ok(r) => {
                self.set_feedback.set(Some(r.error.unwrap_or("Failed".into())));
                self.set_is_err.set(true);
                false
            }
            Err(e) => {
                self.set_feedback.set(Some(e));
                self.set_is_err.set(true);
                false
            }
        }
    }
}

/// Render a list of travelers as `<option>` elements.
pub fn traveler_options(travelers: Vec<TravelerResponse>) -> Vec<impl IntoView> {
    travelers
        .into_iter()
        .map(|t| {
            let display = t.name.clone();
            view! { <option value=t.name>{display}</option> }
        })
        .collect()
}

#[component]
pub fn LoadingCard(message: &'static str) -> impl IntoView {
    view! {
        <div class="card loading-card">
            <div class="spinner"></div>
            <p>{message}</p>
        </div>
    }
}

#[component]
pub fn ErrorCard(message: String) -> impl IntoView {
    view! {
        <div class="card error-card">
            <p class="error-icon">{"\u{26a0}"}</p>
            <p>{message}</p>
        </div>
    }
}

#[component]
pub fn EmptyState(message: &'static str) -> impl IntoView {
    view! {
        <div class="card empty-state">
            <p>{message}</p>
        </div>
    }
}

#[component]
pub fn FeedbackMsg(message: ReadSignal<Option<String>>, is_error: ReadSignal<bool>) -> impl IntoView {
    view! {
        {move || {
            message.get().map(|msg| {
                let cls = if is_error.get() { "feedback error-text" } else { "feedback success-text" };
                view! { <p class=cls>{msg}</p> }
            })
        }}
    }
}
