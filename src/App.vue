
<template>
<div>
  <Landing v-if="page === 'home'" @start="page = 'app'" />
  <AppView v-else />
</div>
</template>

<script setup>
import { ref } from 'vue'
import Landing from './components/Landing.vue'
import AppView from './components/AppView.vue'
const page = ref('home')
</script>
