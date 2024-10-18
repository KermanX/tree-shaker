<script setup lang="ts">
import { ref } from 'vue'

const counter = ref(0)
</script>

<template>
  <div>
    Hello World
  </div>

  <button @click="counter++">{{ counter }}</button>
</template>
