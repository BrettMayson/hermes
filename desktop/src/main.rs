use app::App;
use leptos::{mount::mount_to_body, view, web_sys};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

mod app;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[derive(Debug, serde::Deserialize)]
struct Event<P> {
    payload: P,
}

fn main() {
    console_error_panic_hook::set_once();

    spawn_local(async move {
        let closure: Closure<dyn FnMut(JsValue)> =
            Closure::<dyn FnMut(_)>::new(move |s: JsValue| {
                let event: Event<String> = serde_wasm_bindgen::from_value(s).unwrap();
                if event.payload == "dark" {
                    web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .document_element()
                        .unwrap()
                        .set_attribute("data-theme", "dark")
                        .unwrap();
                } else {
                    web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .document_element()
                        .unwrap()
                        .set_attribute("data-theme", "light")
                        .unwrap();
                }
            });
        listen("theme", closure.as_ref().unchecked_ref()).await;
        closure.forget();
    });

    mount_to_body(|| {
        view! { <App/> }
    })
}
