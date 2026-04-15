use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn DashboardPage(init_data: String, refresh: u32) -> impl IntoView {
    let _ = refresh;
    let (data, set_data) = signal(Option::<Result<SummaryResponse, String>>::None);

    spawn_local({
        let init_data = init_data.clone();
        async move {
            set_data.set(Some(api::fetch_summary(&init_data).await));
        }
    });

    view! {
        <div class="page">
            {move || match data.get() {
                None => view! { <LoadingCard message="Loading summary..." /> }.into_any(),
                Some(Err(e)) => view! { <ErrorCard message=e /> }.into_any(),
                Some(Ok(s)) => view! {
                    <div class="card">
                        <h2>"Trip Overview"</h2>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-value">{s.traveler_count.to_string()}</span>
                                <span class="stat-label">"Travelers"</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-value">{s.expense_count.to_string()}</span>
                                <span class="stat-label">"Expenses"</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-value">{s.transfer_count.to_string()}</span>
                                <span class="stat-label">"Transfers"</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-value">
                                    {format!("{} {}", s.total_expenses, s.currency)}
                                </span>
                                <span class="stat-label">"Total spent"</span>
                            </div>
                        </div>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
