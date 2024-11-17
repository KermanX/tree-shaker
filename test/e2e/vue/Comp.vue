<script setup lang="ts">
const props = defineProps<{
  msg: string
}>()
const name = defineModel<string>()
</script>

<template>
  <label>
    {{ props.msg }}
    <input v-model="name" type="text" />
  </label>
</template>
