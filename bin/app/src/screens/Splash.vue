<script>
import { emit } from '@tauri-apps/api/event';
import { onMounted } from 'vue';

export default {
  mounted() {
    emit('global:awake');
    console.log(`the component is now mounted.`)
  }
}

</script>

<template>
  <div class="loading">
    <img src="../assets/harmony.png" />
  </div>
  <div class="splash">
  </div>
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
