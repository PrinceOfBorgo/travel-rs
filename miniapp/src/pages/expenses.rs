use leptos::prelude::*;
use std::collections::BTreeMap;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn ExpensesPage(
    init_data: String,
    refresh: u32,
    set_refresh: WriteSignal<u32>,
) -> impl IntoView {
    let _ = refresh;
    let (data, set_data) = signal(Option::<Result<Vec<ExpenseResponse>, String>>::None);
    let (travelers, set_travelers) = signal(Vec::<TravelerResponse>::new());
    let (show_form, set_show_form) = signal(false);
    let (description, set_description) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (paid_by, set_paid_by) = signal(String::new());
    let (split_mode, set_split_mode) = signal("equal".to_string());
    let form = FormState::new();

    {
        let init_data = init_data.clone();
        spawn_local(async move {
            set_data.set(Some(api::fetch_expenses(&init_data).await));
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
            let desc = description.get();
            let amt = amount.get();
            let payer = paid_by.get();
            if desc.trim().is_empty() || amt.trim().is_empty() || payer.trim().is_empty() {
                form.set_feedback.set(Some("Fill in all fields".into()));
                form.set_is_err.set(true);
                return;
            }
            form.set_submitting.set(true);
            form.set_feedback.set(None);

            let mode = split_mode.get();
            let travelers_list = travelers.get();
            let shares: BTreeMap<String, String> = if mode == "equal" {
                travelers_list
                    .iter()
                    .map(|t| (t.name.clone(), "equal".to_string()))
                    .collect()
            } else {
                std::iter::once((payer.clone(), amt.clone())).collect()
            };

            let req = AddExpenseRequest {
                description: desc,
                amount: amt,
                paid_by: payer,
                shares,
            };
            let init_data = init_data.clone();
            spawn_local(async move {
                let result = api::add_expense(&init_data, &req).await;
                if form.handle(result, "Expense added".into()) {
                    set_description.set(String::new());
                    set_amount.set(String::new());
                    set_paid_by.set(String::new());
                    set_show_form.set(false);
                    set_refresh.update(|v| *v += 1);
                    set_data.set(Some(api::fetch_expenses(&init_data).await));
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
                let result = api::delete_expense(&init_data, number).await;
                if form.handle(result, format!("Expense #{number} deleted")) {
                    set_refresh.update(|v| *v += 1);
                    set_data.set(Some(api::fetch_expenses(&init_data).await));
                }
            });
        }
    };

    view! {
        <div class="page">
            <div class="card">
                <div class="card-header-row">
                    <h2>"Expenses"</h2>
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| set_show_form.update(|v| *v = !*v)
                    >
                        {move || if show_form.get() { "Cancel" } else { "+ New" }}
                    </button>
                </div>

                {move || show_form.get().then(|| {
                    let travelers_list = travelers.get();
                    view! {
                        <div class="form-section">
                            <input
                                class="form-input"
                                type="text"
                                placeholder="Description"
                                prop:value=move || description.get()
                                on:input=move |ev| set_description.set(event_target_value(&ev))
                            />
                            <input
                                class="form-input"
                                type="number"
                                placeholder="Amount"
                                step="0.01"
                                prop:value=move || amount.get()
                                on:input=move |ev| set_amount.set(event_target_value(&ev))
                            />
                            <select
                                class="form-input"
                                on:change=move |ev| set_paid_by.set(event_target_value(&ev))
                            >
                                <option value="" disabled=true selected=true>"Paid by..."</option>
                                {traveler_options(travelers_list)}
                            </select>
                            <select
                                class="form-input"
                                on:change=move |ev| set_split_mode.set(event_target_value(&ev))
                            >
                                <option value="equal" selected=true>"Split equally among all"</option>
                                <option value="payer">"Payer only"</option>
                            </select>
                            <button
                                class="btn btn-primary"
                                on:click=on_add.clone()
                                disabled=move || form.submitting.get()
                            >
                                {move || if form.submitting.get() { "Saving..." } else { "Add Expense" }}
                            </button>
                        </div>
                    }
                })}
                <FeedbackMsg message=form.feedback is_error=form.is_err />
            </div>

            {move || match data.get() {
                None => view! { <LoadingCard message="Loading expenses..." /> }.into_any(),
                Some(Err(e)) => view! { <ErrorCard message=e /> }.into_any(),
                Some(Ok(items)) if items.is_empty() => {
                    view! { <EmptyState message="No expenses yet." /> }.into_any()
                }
                Some(Ok(items)) => {
                    let on_del = on_delete.clone();
                    view! {
                        <div class="card">
                            <ul class="list">
                                {items
                                    .into_iter()
                                    .map(|e| {
                                        let num = e.number;
                                        let del = on_del.clone();
                                        view! {
                                            <li class="list-item expense-item">
                                                <div class="item-main">
                                                    <span class="item-number">{"#"}{e.number.to_string()}</span>
                                                    <span class="item-description">{e.description}</span>
                                                </div>
                                                <span class="item-amount">{e.amount}</span>
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
