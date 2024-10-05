<script setup lang="ts">
import { watchEffect } from 'vue';
import { activeLogIndex, logsRaw } from './states';

function getClass(index: number) {
  const active = index === activeLogIndex.value;
  return {
    'text-gray-200 bg-gray-800 bg-op-60': index < activeLogIndex.value,
    'bg-gray-600 bg-op-60': index === activeLogIndex.value,
    'text-gray-400 bg-gray-700 bg-op-0': index > activeLogIndex.value,
  };
}

function next() {
  activeLogIndex.value++;
}

function prev() {
  activeLogIndex.value--;
}

watchEffect(() => {
  if (activeLogIndex.value >= logsRaw.value.length) {
    activeLogIndex.value = logsRaw.value.length - 1;
  }
  if (activeLogIndex.value < 0) {
    activeLogIndex.value = 0;
  }
});
</script>

<template>
  <div flex flex-col gap-y-1 text-sm>
    <div font-mono flex-grow h-0 overflow-auto scroll-hidden class="no-scrollbar">
      <div my-2>
        <div v-for="log, index in logsRaw" flex class="hover:bg-op-100" :class="getClass(index)" @click="activeLogIndex = index">
          <div w-2rem>

          </div>
          <div flex-grow>
            {{ log }}
          </div>
        </div>
      </div>
    </div>

    <div flex gap-x-2 b-t-1 border-gray-600 b-solid pt-3 px-1>
      <div flex-grow />
      <button @click="prev" :disabled="activeLogIndex <= 0">Prev</button>
      <button @click="next" :disabled="activeLogIndex >= logsRaw.length - 1">Next</button>
    </div>
  </div>
</template>

<style scoped>
button {
  @apply px-2 py-.5 rounded text-white b-1 b-gray-400 hover:bg-white hover:bg-op-10 active:bg-op-20 disabled:op-40 disabled:cursor-not-allowed disabled:pointer-events-none disabled:user-select-none user-select-none;
}
</style>
