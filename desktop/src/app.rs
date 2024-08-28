use std::path::PathBuf;

use leptos::*;
use serde::{Deserialize, Serialize};
use hermes_desktop_comm::setup::{Platform, Setup};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::{invoke, listen, Event};


#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    // let greet = move |ev: SubmitEvent| {
    //     ev.prevent_default();
    //     spawn_local(async move {
    //         let name = name.get_untracked();
    //         if name.is_empty() {
    //             return;
    //         }

    //         let args = to_value(&GreetArgs { name: &name }).unwrap();
    //         // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    //         let new_msg = invoke("greet", args).await.as_string().unwrap();
    //         set_greet_msg.set(new_msg);
    //     });
    // };

    let (connected, setConnected) = create_signal(false);
    let (platform, setPlatform) = create_signal(None::<Platform>);
    let (arma3dir, setArma3Dir) = create_signal(None::<PathBuf>);

    spawn_local(async move {
        let closure = Closure::<dyn FnMut(_)>::new(move |_: JsValue| {
            setConnected.set(true);
        });
        listen("connected", closure.as_ref().unchecked_ref()).await;
        closure.forget();
    });
    spawn_local(async move {
        let closure = Closure::<dyn FnMut(_)>::new(move |event: JsValue| {
            logging::log!("event: {:?}", event);
            let event: Event<Setup> = serde_wasm_bindgen::from_value(event).unwrap();
            setPlatform.set(Some(event.payload.platform));
            setArma3Dir.set(event.payload.arma_3_location);
        });
        listen("setup", closure.as_ref().unchecked_ref()).await;
        closure.forget();
    });

    spawn_local(async move {
        invoke("init_process", JsValue::NULL).await;
    });

    view! {
        <main class="app-mainbackground flex flex-col items-center justify-center h-screen w-screen">
            <div class="card bg-base-100 w-96">
                <div class="card-body">
                    <h2 class="card-title">Hermes is Starting</h2>
                    <ul class="steps steps-vertical">
                        <li
                            data-content=move || if connected.get() { "✓" } else { "•" }
                            class="step"
                            class:step-primary=move || connected.get()
                        >
                            Systems Init
                        </li>
                        <li
                            data-content=move || {
                                if platform.get().is_some() {
                                    if arma3dir.get().is_some() { "✓" } else { "?" }
                                } else {
                                    "•"
                                }
                            }

                            class="step"
                            class:step-primary=move || {
                                platform.get().is_some() && arma3dir.get().is_some()
                            }

                            class:step-secondary=move || {
                                platform.get().is_some() && arma3dir.get().is_none()
                            }
                        >

                            <div>
                                {move || {
                                    format!(
                                        "Arma 3 {}",
                                        match platform.get() {
                                            Some(Platform::Windows) => "- Windows",
                                            Some(Platform::LinuxNative) => "- Linux (Native)",
                                            Some(Platform::LinuxFlatpak) => "- Linux (Flatpak)",
                                            None => "",
                                        },
                                    )
                                }}

                            </div>
                        </li>
                        <li data-content="•" class="step">
                            Existing Repos
                        </li>
                        <li data-content="•" class="step">
                            Ready
                        </li>
                    </ul>
                </div>
            </div>
        </main>
    }
}
