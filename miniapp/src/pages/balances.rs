use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn BalancesPage(init_data: String, refresh: u32) -> impl IntoView {
    let _ = refresh;
    let (data, set_data) = signal(Option::<Result<Vec<BalanceResponse>, String>>::None);

    spawn_local({
        let init_data = init_data.clone();
        async move {
            set_data.set(Some(api::fetch_balances(&init_data).await));
        }
    });

    view! {
        <div class="page">
            {move || match data.get() {
                None => view! { <LoadingCard message="Loading balances..." /> }.into_any(),
                Some(Err(e)) => view! { <ErrorCard message=e /> }.into_any(),
                Some(Ok(items)) if items.is_empty() => {
                    view! { <EmptyState message="All settled up!" /> }.into_any()
                }
                Some(Ok(items)) => view! {
                    <div class="card">
                        <h2>"Balances"</h2>
                        <ul class="list">
                            {items
                                .into_iter()
                                .map(|b| view! {
                                    <li class="list-item balance-item">
                                        <div class="item-main">
                                            <span class="balance-debtor">{b.debtor_name.clone()}</span>
                                            <span class="balance-arrow">{" owes "}</span>
                                            <span class="balance-creditor">{b.creditor_name.clone()}</span>
                                        </div>
                                        <span class="item-amount balance-debt">{b.debt}</span>
                                    </li>
                                })
                                .collect::<Vec<_>>()}
                        </ul>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
