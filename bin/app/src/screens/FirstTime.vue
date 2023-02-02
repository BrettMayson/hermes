<script setup lang="ts">
import DirectorySelect from '@/components/forms/DirectorySelect.vue';
import { useStore } from '@/store';

const store = useStore();

const updateArma3Folder = (folder: string) => {
  store.state.root.arma3folder = folder;
}

const updateDepotFolder = (folder: string) => {
  store.state.root.depotfolder = folder;
}
</script>

<template>
  <div class="loading">
    <div class="space-y-6 sm:px-6 lg:col-span-9 lg:px-0">
      <form action="#" method="POST">
        <div class="shadow sm:overflow-hidden sm:rounded-md">
          <div class="space-y-6 bg-white py-6 px-4 sm:p-6">
            <div>
              <h3 class="text-lg font-medium leading-6 text-gray-900">Directory Setup</h3>
            </div>

            <DirectorySelect label="Arma 3 Install" description="Where Arma 3 is installed via Steam." placeholder="C:\?" :value="store.state.root.arma3folder" @update="updateArma3Folder"/>
            <DirectorySelect label="Mod Depot" description="Where Harmony will keep mods. Select an empty folder on a drive with a lot of free space." placeholder="C:\?" :value="store.state.root.depotfolder" @update="updateDepotFolder"/>
          </div>
          <div class="bg-gray-50 px-4 py-3 text-right sm:px-6">
            <button type="submit"
              class="inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2">Save</button>
          </div>
        </div>
      </form>
    </div>
  </div>
  <div class="splash"></div>
</template>

<style scoped>
.loading {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
}

.loading img {
  width: 200px;
  height: 200px;
}

.splash {
  top: 0;
  left: 0;
  z-index: -1;
  position: absolute;
  height: 100vh;
  width: 100vw;
}

.splash::after {
  content: "";
  height: 100vh;
  width: 100vw;
  position: absolute;
  background-image: url('../assets/splash_1.jpg');
  background-repeat: no-repeat;
  background-size: cover;
  background-position: center;
}
</style>
