use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Safe wrapper around the Telegram WebApp JavaScript API.
/// Always reads `initData` (the signed string), never trusts `initDataUnsafe`.
pub struct TelegramWebApp {
    webapp: Option<JsValue>,
}

impl TelegramWebApp {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global window");
        let telegram = js_sys::Reflect::get(&window, &JsValue::from_str("Telegram"))
            .ok()
            .filter(|v| !v.is_undefined());
        let webapp = telegram.and_then(|tg| {
            js_sys::Reflect::get(&tg, &JsValue::from_str("WebApp"))
                .ok()
                .filter(|v| !v.is_undefined())
        });

        // Expand the Mini App to full height if available
        if let Some(ref wa) = webapp {
            if let Ok(f) = js_sys::Reflect::get(wa, &JsValue::from_str("expand")) {
                if let Some(func) = f.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(wa);
                }
            }
        }

        // Signal to Telegram that the Mini App is ready
        if let Some(ref wa) = webapp {
            if let Ok(f) = js_sys::Reflect::get(wa, &JsValue::from_str("ready")) {
                if let Some(func) = f.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(wa);
                }
            }
        }

        Self { webapp }
    }

    /// Returns the raw `initData` string (signed by Telegram).
    /// This is the ONLY value safe to send to the server for validation.
    pub fn init_data(&self) -> Option<String> {
        self.webapp.as_ref().and_then(|wa| {
            js_sys::Reflect::get(wa, &JsValue::from_str("initData"))
                .ok()
                .and_then(|v| v.as_string())
        })
    }

    /// Returns the user's first name from `initDataUnsafe` for display purposes only.
    /// This MUST NOT be used for authentication — always validate `initData` on the server.
    pub fn user_first_name(&self) -> Option<String> {
        self.webapp.as_ref().and_then(|wa| {
            let unsafe_data =
                js_sys::Reflect::get(wa, &JsValue::from_str("initDataUnsafe")).ok()?;
            let user = js_sys::Reflect::get(&unsafe_data, &JsValue::from_str("user")).ok()?;
            let first_name = js_sys::Reflect::get(&user, &JsValue::from_str("first_name")).ok()?;
            first_name.as_string()
        })
    }
}
