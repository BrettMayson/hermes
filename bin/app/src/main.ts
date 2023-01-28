import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";

createApp(App).mount("#app");

import { emit, listen } from '@tauri-apps/api/event'
import { open, message } from '@tauri-apps/api/dialog'

// Rust couldn't find Arma 3, ask the user for the folder
await listen('global:arma3folder', (event) => {
  async function askForArma3Folder() {
    await message('Harmony was unable to find your Arma 3 game directory. You will need to select it. If you do not have Arma installed yet, install it via Steam and launch it once before using Harmony.', { title: 'Harmony', type: 'error' });
    const selected = await open({
      directory: true,
      title: 'Select Arma 3 folder',
    });
    if (selected) {
      emit('global:arma3folder:ok', selected);
    } else {
      emit('global:arma3folder:cancel');
    }
  }
  askForArma3Folder();
})

emit('global:log', 'Hello from Vue');
