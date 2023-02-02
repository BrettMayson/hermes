import { createApp } from "vue";
import { store, key, RootConfig } from "./store";

import App from "./App.vue";
import "./style.css";
import "./tailwind.css"

createApp(App).use(store, key).mount("#app");

import { emit, listen } from "@tauri-apps/api/event";
import { open, message } from "@tauri-apps/api/dialog";

await listen<[boolean, RootConfig]>("global:RootConfigLoad", (event) => {
  console.log("global:RootConfigLoad", event);
  store.state.received = true;
  store.state.firstTime = event.payload[0];
  store.state.root = event.payload[1];
});

emit("global:log", "Hello from Vue");
