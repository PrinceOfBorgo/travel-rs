use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn SettingsPage(
    init_data: String,
    refresh: u32,
    set_refresh: WriteSignal<u32>,
) -> impl IntoView {
    let _ = refresh;
    let (settings, set_settings) = signal(Option::<Result<SettingsResponse, String>>::None);
    let (currency, set_currency) = signal(String::new());
    let (language, set_language) = signal(String::new());
    let form = FormState::new();

    {
        let init_data = init_data.clone();
        spawn_local(async move {
            match api::fetch_settings(&init_data).await {
                Ok(s) => {
                    set_currency.set(s.currency.clone());
                    set_language.set(s.language.clone());
                    set_settings.set(Some(Ok(s)));
                }
                Err(e) => set_settings.set(Some(Err(e))),
            }
        });
    }

    let on_save_currency = {
        let init_data = init_data.clone();
        move |_| {
            if form.submitting.get() {
                return;
            }
            let c = currency.get();
            if c.trim().is_empty() {
                return;
            }
            form.set_submitting.set(true);
            form.set_feedback.set(None);
            let init_data = init_data.clone();
            spawn_local(async move {
                let result = api::set_currency(&init_data, &c).await;
                if form.handle(result, format!("Currency set to {c}")) {
                    set_refresh.update(|v| *v += 1);
                }
                form.set_submitting.set(false);
            });
        }
    };

    let on_save_language = {
        let init_data = init_data.clone();
        move |_| {
            if form.submitting.get() {
                return;
            }
            let l = language.get();
            if l.trim().is_empty() {
                return;
            }
            form.set_submitting.set(true);
            form.set_feedback.set(None);
            let init_data = init_data.clone();
            spawn_local(async move {
                let result = api::set_language(&init_data, &l).await;
                if form.handle(result, format!("Language set to {l}")) {
                    set_refresh.update(|v| *v += 1);
                }
                form.set_submitting.set(false);
            });
        }
    };

    view! {
        <div class="page">
            {move || match settings.get() {
                None => view! { <LoadingCard message="Loading settings..." /> }.into_any(),
                Some(Err(e)) => view! { <ErrorCard message=e /> }.into_any(),
                Some(Ok(_)) => view! {
                    <div class="card">
                        <h2>"Currency"</h2>
                        <div class="form-row">
                            <input
                                class="form-input"
                                type="text"
                                placeholder="e.g. EUR, USD"
                                prop:value=move || currency.get()
                                on:input=move |ev| set_currency.set(event_target_value(&ev))
                            />
                            <button
                                class="btn btn-primary"
                                on:click=on_save_currency.clone()
                                disabled=move || form.submitting.get()
                            >"Save"</button>
                        </div>
                    </div>
                    <div class="card">
                        <h2>"Language"</h2>
                        <div class="form-row">
                            <input
                                class="form-input"
                                type="text"
                                placeholder="e.g. en-US, it-IT"
                                prop:value=move || language.get()
                                on:input=move |ev| set_language.set(event_target_value(&ev))
                            />
                            <button
                                class="btn btn-primary"
                                on:click=on_save_language.clone()
                                disabled=move || form.submitting.get()
                            >"Save"</button>
                        </div>
                        <FeedbackMsg message=form.feedback is_error=form.is_err />
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
