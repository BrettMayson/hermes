<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog'

const props = defineProps<{
  label: string;
  placeholder: string;
  description: string;
  value: string | null;
}>();

const emit = defineEmits(['update'])

async function browse() {
  const selected = await open({
    directory: true,
    title: props.label,
    defaultPath: props.value || "",
  });
  if (selected)
    emit('update', selected);
};

</script>

<template>
  <div class="col-span-6 sm:col-span-3">
    <label for="first-name" class="block text-sm font-medium text-gray-700">{{ props.label }}</label>
    <div class="mt-1 flex rounded-md shadow-sm">
      <div class="relative flex flex-grow items-stretch focus-within:z-10">
        <input :value="props.value" type="text"
          class="block w-full rounded-none rounded-l-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          :placeholder="placeholder" />
      </div>
      <button type="button"
        class="relative -ml-px inline-flex items-center space-x-2 rounded-none rounded-r-md border border-gray-300 bg-gray-50 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
        @click="browse">
        <span>Browse</span>
      </button>
    </div>
    <p class="mt-2 text-sm text-gray-500" id="email-description">{{ props.description }}</p>
  </div>
</template>
