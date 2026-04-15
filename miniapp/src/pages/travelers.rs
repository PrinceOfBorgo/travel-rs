use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn TravelersPage(
    init_data: String,
    refresh: u32,
    set_refresh: WriteSignal<u32>,
) -> impl IntoView {
    let _ = refresh;
    let (data, set_data) = signal(Option::<Result<Vec<TravelerResponse>, String>>::None);
    let (new_name, set_new_name) = signal(String::new());
    let form = FormState::new();

    {
        let init_data = init_data.clone();
        spawn_local(async move {
            set_data.set(Some(api::fetch_travelers(&init_data).await));
        });
    }

    let on_add = {
        let init_data = init_data.clone();
        move |_| {
            let name = new_name.get();
            if name.trim().is_empty() || form.submitting.get() {
                return;
            }
            form.set_submitting.set(true);
            form.set_feedback.set(None);
            let init_data = init_data.clone();
            spawn_local(async move {
                let result = api::add_traveler(&init_data, &name).await;
                if form.handle(result, format!("Added {name}")) {
                    set_new_name.set(String::new());
                    set_refresh.update(|v| *v += 1);
                    set_data.set(Some(api::fetch_travelers(&init_data).await));
                }
                form.set_submitting.set(false);
            });
        }
    };

    let on_delete = {
        let init_data = init_data.clone();
        move |name: String| {
            let init_data = init_data.clone();
            form.set_feedback.set(None);
            spawn_local(async move {
                let result = api::delete_traveler(&init_data, &name).await;
                if form.handle(result, format!("Deleted {name}")) {
                    set_refresh.update(|v| *v += 1);
                    set_data.set(Some(api::fetch_travelers(&init_data).await));
                }
            });
        }
    };

    view! {
        <div class="page">
            <div class="card">
                <h2>"Add Traveler"</h2>
                <div class="form-row">
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Name"
                        prop:value=move || new_name.get()
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                    <button
                        class="btn btn-primary"
                        on:click=on_add
                        disabled=move || form.submitting.get()
                    >
                        {move || if form.submitting.get() { "..." } else { "Add" }}
                    </button>
                </div>
                <FeedbackMsg message=form.feedback is_error=form.is_err />
            </div>

            {move || match data.get() {
                None => view! { <LoadingCard message="Loading travelers..." /> }.into_any(),
                Some(Err(e)) => view! { <ErrorCard message=e /> }.into_any(),
                Some(Ok(items)) if items.is_empty() => {
                    view! { <EmptyState message="No travelers yet." /> }.into_any()
                }
                Some(Ok(items)) => {
                    let on_del = on_delete.clone();
                    view! {
                        <div class="card">
                            <h2>"Travelers"</h2>
                            <ul class="list">
                                {items
                                    .into_iter()
                                    .map(|t| {
                                        let name = t.name.clone();
                                        let del = on_del.clone();
                                        view! {
                                            <li class="list-item">
                                                <span class="item-icon">{"\u{1f464}"}</span>
                                                <span class="flex-1">{t.name.clone()}</span>
                                                <button
                                                    class="btn btn-danger btn-sm"
                                                    on:click=move |_| del(name.clone())
                                                >{"\u{1f5d1}"}</button>
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </ul>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
