<script setup lang="ts">
import Splash from "./screens/Splash.vue";
import FirstTime from "./screens/FirstTime.vue";
import { useStore } from "./store";

const store = useStore();
</script>

<template>
  <template v-if="!store.state.received">
    <Splash />
  </template>
  <template v-else>
    <template v-if="store.state.firstTime">
      <FirstTime />
    </template>
  </template>
</template>
