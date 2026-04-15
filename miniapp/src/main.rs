mod api;
mod components;
mod pages;
mod telegram;
mod types;

use leptos::prelude::*;
use telegram::TelegramWebApp;
use wasm_bindgen_futures::spawn_local;

use components::*;
use pages::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

// ── Navigation ──

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Dashboard,
    Travelers,
    Expenses,
    Transfers,
    Balances,
    Settings,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Overview",
            Self::Travelers => "Travelers",
            Self::Expenses => "Expenses",
            Self::Transfers => "Transfers",
            Self::Balances => "Balances",
            Self::Settings => "Settings",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "\u{1f4ca}",
            Self::Travelers => "\u{1f465}",
            Self::Expenses => "\u{1f4b3}",
            Self::Transfers => "\u{1f4b8}",
            Self::Balances => "\u{2696}",
            Self::Settings => "\u{2699}",
        }
    }
}

const NAV_PAGES: [Page; 6] = [
    Page::Dashboard,
    Page::Travelers,
    Page::Expenses,
    Page::Transfers,
    Page::Balances,
    Page::Settings,
];

// ── App shell ──

#[component]
fn App() -> impl IntoView {
    let (page, set_page) = signal(Page::Dashboard);
    let (auth_error, set_auth_error) = signal(Option::<String>::None);
    let (init_data, set_init_data) = signal(Option::<String>::None);
    let (user_name, set_user_name) = signal(String::new());
    let (authenticated, set_authenticated) = signal(false);
    let (refresh, set_refresh) = signal(0u32);

    let webapp = TelegramWebApp::new();

    if let Some(name) = webapp.user_first_name() {
        set_user_name.set(name);
    }

    match webapp.init_data() {
        Some(data) if !data.is_empty() => {
            let data_clone = data.clone();
            set_init_data.set(Some(data));

            spawn_local(async move {
                match api::create_session(&data_clone).await {
                    Ok(resp) if resp.ok => {
                        set_authenticated.set(true);
                    }
                    Ok(resp) => {
                        set_auth_error
                            .set(Some(resp.error.unwrap_or("Authentication failed".into())));
                    }
                    Err(e) => {
                        set_auth_error.set(Some(e));
                    }
                }
            });
        }
        _ => {
            set_auth_error.set(Some("Not running inside Telegram".into()));
        }
    }

    view! {
        <div class="app">
            <style>{include_str!("style.css")}</style>

            <header class="app-header">
                <h1>"TravelRS"</h1>
                <p class="subtitle">
                    {move || {
                        let name = user_name.get();
                        if name.is_empty() {
                            "Travel expense tracker".to_string()
                        } else {
                            format!("Welcome, {name}")
                        }
                    }}
                </p>
            </header>

            <main class="app-content">
                {move || {
                    if let Some(err) = auth_error.get() {
                        view! { <ErrorCard message=err /> }.into_any()
                    } else if !authenticated.get() {
                        view! { <LoadingCard message="Authenticating..." /> }.into_any()
                    } else if let Some(data) = init_data.get() {
                        let r = refresh.get();
                        let current = page.get();
                        match current {
                            Page::Dashboard => view! { <DashboardPage init_data=data.clone() refresh=r /> }.into_any(),
                            Page::Travelers => view! { <TravelersPage init_data=data.clone() refresh=r set_refresh=set_refresh /> }.into_any(),
                            Page::Expenses => view! { <ExpensesPage init_data=data.clone() refresh=r set_refresh=set_refresh /> }.into_any(),
                            Page::Transfers => view! { <TransfersPage init_data=data.clone() refresh=r set_refresh=set_refresh /> }.into_any(),
                            Page::Balances => view! { <BalancesPage init_data=data.clone() refresh=r /> }.into_any(),
                            Page::Settings => view! { <SettingsPage init_data=data.clone() refresh=r set_refresh=set_refresh /> }.into_any(),
                        }
                    } else {
                        view! { <ErrorCard message="No auth data available".to_string() /> }.into_any()
                    }
                }}
            </main>

            <nav class="bottom-nav">
                {NAV_PAGES
                    .into_iter()
                    .map(|p| {
                        let active = move || page.get() == p;
                        view! {
                            <button
                                class="nav-item"
                                class:active=active
                                on:click=move |_| set_page.set(p)
                            >
                                <span class="nav-icon">{p.icon()}</span>
                                <span class="nav-label">{p.label()}</span>
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </nav>
        </div>
    }
}
