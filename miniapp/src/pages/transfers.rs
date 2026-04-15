use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn TransfersPage(
    init_data: String,
    refresh: u32,
    set_refresh: WriteSignal<u32>,
) -> impl IntoView {
    let _ = refresh;
    let (data, set_data) = signal(Option::<Result<Vec<TransferResponse>, String>>::None);
    let (travelers, set_travelers) = signal(Vec::<TravelerResponse>::new());
    let (show_form, set_show_form) = signal(false);
    let (from, set_from) = signal(String::new());
    let (to, set_to) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let form = FormState::new();

    {
        let init_data = init_data.clone();
        spawn_local(async move {
            set_data.set(Some(api::fetch_transfers(&init_data).await));
            if let Ok(t) = api::fetch_travelers(&init_data).await {
                set_travelers.set(t);
            }
        });
    }

    let on_add = {
        let init_data = init_data.clone();
        move |_| {
            if form.submitting.get() {
                return;
            }
            let f = from.get();
            let t = to.get();
            let a = amount.get();
            if f.trim().is_empty() || t.trim().is_empty() || a.trim().is_empty() {
                form.set_feedback.set(Some("Fill in all fields".into()));
                form.set_is_err.set(true);
                return;
            }
            if f == t {
                form.set_feedback.set(Some("Sender and receiver must differ".into()));
                form.set_is_err.set(true);
                return;
            }
            form.set_submitting.set(true);
            form.set_feedback.set(None);
            let init_data = init_data.clone();
            spawn_local(async move {
                let result = api::add_transfer(&init_data, &f, &t, &a).await;
                if form.handle(result, "Transfer added".into()) {
                    set_from.set(String::new());
                    set_to.set(String::new());
                    set_amount.set(String::new());
                    set_show_form.set(false);
                    set_refresh.update(|v| *v += 1);
                    set_data.set(Some(api::fetch_transfers(&init_data).await));
                }
                form.set_submitting.set(false);
            });
        }
    };

    let on_delete = {
        let init_data = init_data.clone();
        move |number: i64| {
            let init_data = init_data.clone();
            form.set_feedback.set(None);
            spawn_local(async move {
                let result = api::delete_transfer(&init_data, number).await;
                if form.handle(result, format!("Transfer #{number} deleted")) {
                    set_refresh.update(|v| *v += 1);
                    set_data.set(Some(api::fetch_transfers(&init_data).await));
                }
            });
        }
    };

    view! {
        <div class="page">
            <div class="card">
                <div class="card-header-row">
                    <h2>"Transfers"</h2>
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| set_show_form.update(|v| *v = !*v)
                    >
                        {move || if show_form.get() { "Cancel" } else { "+ New" }}
                    </button>
                </div>

                {move || show_form.get().then(|| {
                    let travelers_list = travelers.get();
                    let travelers_list2 = travelers_list.clone();
                    view! {
                        <div class="form-section">
                            <select
                                class="form-input"
                                on:change=move |ev| set_from.set(event_target_value(&ev))
                            >
                                <option value="" disabled=true selected=true>"From..."</option>
                                {traveler_options(travelers_list)}
                            </select>
                            <select
                                class="form-input"
                                on:change=move |ev| set_to.set(event_target_value(&ev))
                            >
                                <option value="" disabled=true selected=true>"To..."</option>
                                {traveler_options(travelers_list2)}
                            </select>
                            <input
                                class="form-input"
                                type="number"
                                placeholder="Amount"
                                step="0.01"
                                prop:value=move || amount.get()
                                on:input=move |ev| set_amount.set(event_target_value(&ev))
                            />
                            <button
                                class="btn btn-primary"
                                on:click=on_add.clone()
                                disabled=move || form.submitting.get()
                            >
                                {move || if form.submitting.get() { "Saving..." } else { "Add Transfer" }}
                            </button>
                        </div>
                    }
                })}
                <FeedbackMsg message=form.feedback is_error=form.is_err />
            </div>

            {move || match data.get() {
                None => view! { <LoadingCard message="Loading transfers..." /> }.into_any(),
                Some(Err(e)) => view! { <ErrorCard message=e /> }.into_any(),
                Some(Ok(items)) if items.is_empty() => {
                    view! { <EmptyState message="No transfers yet." /> }.into_any()
                }
                Some(Ok(items)) => {
                    let on_del = on_delete.clone();
                    view! {
                        <div class="card">
                            <ul class="list">
                                {items
                                    .into_iter()
                                    .map(|t| {
                                        let num = t.number;
                                        let del = on_del.clone();
                                        view! {
                                            <li class="list-item transfer-item">
                                                <div class="item-main">
                                                    <span class="item-number">{"#"}{t.number.to_string()}</span>
                                                    <span class="item-description">
                                                        {format!("{} \u{2192} {}", t.sender_name, t.receiver_name)}
                                                    </span>
                                                </div>
                                                <span class="item-amount">{t.amount}</span>
                                                <button
                                                    class="btn btn-danger btn-sm"
                                                    on:click=move |_| del(num)
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
