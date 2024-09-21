<script setup lang="ts">
import { compressToBase64, decompressFromBase64 } from 'lz-string'
import { tree_shake } from '@kermanx/tree-shaker'
import { computed, ref, watch, watchEffect } from 'vue'
import Editor from './Editor.vue'
import { DEMO } from './examples';

const input = ref('')
const doTreeShake = ref(true)
const doMinify = ref(false)

const debouncedInput = ref('')
let debounceTimeout = Number.NaN
watch(input, (input) => {
  clearInterval(debounceTimeout)
  debounceTimeout = setTimeout(() => {
    debouncedInput.value = input
  }, 300)
})

function load() {
  let parsed
  if (window.location.hash) {
    try {
      parsed = JSON.parse(decompressFromBase64(window.location.hash.slice(1)) || '{}')
    }
    catch (e) { console.error(e) }
  }
  parsed ||= {}
  debouncedInput.value = input.value = parsed.input ?? DEMO
  doTreeShake.value = parsed.doTreeShake ?? true
  doMinify.value = parsed.doMinify ?? false
}

function save() {
  window.location.hash = compressToBase64(JSON.stringify({
    input: input.value,
    doTreeShake: doTreeShake.value,
    doMinify: doMinify.value,
  }))
}

load()
watchEffect(save)

const result = computed(() =>
  tree_shake(debouncedInput.value, doTreeShake.value, doMinify.value, false).trim()
  || `// Empty output or error`
)
</script>

<template>
  <div py-2 md:py-4 fixed inset-0 flex flex-col>
    <div px-4 flex flex-wrap gap-x-2 pb-2>
      <h1 text-xl md:text-3xl font-bold md:pb-2 select-none flex flex-wrap items-center gap-x-2>
        <img src="/favicon.ico" h-1em bg-gray-200 rounded-lg>
        Tree Shaker
        <div text-sm self-end flex items-center gap-1 op-80>
          Experimental Tree Shaker for JS Based on Oxc
          <a i-carbon-logo-github flex-grow inline-block w-1.2em h-1.2em hover:op-80 href="https://github.com/KermanX/tree-shaker" target="_blank" />
        </div>
      </h1>
      <div flex-grow />
      <div flex w-fit md:flex-col h-min md:h-0 z-10 gap-x-2 gap-y-1 font-mono items-end mr-2>
        <label flex align-center gap-1 select-none>
          <span op-80>
            Do tree shake:
          </span>
          <input v-model="doTreeShake" type="checkbox" placeholder="ast_builder">
        </label>
        <label flex align-center gap-1 select-none>
          <span op-80>
            Do minify:
          </span>
          <input v-model="doMinify" type="checkbox">
        </label>
      </div>
    </div>
    <div flex-grow flex flex-col md:flex-row gap-x-6 gap-y-2>
      <div flex-grow h-0 md:h-full md:w-0 flex flex-col>
        <div flex items-center>
          <h2 md:text-xl pb-2 pl-4 select-none>
            Input
          </h2>
        </div>
        <Editor v-model="input" lang="javascript" class="flex-grow h-0 max-h-full" />
      </div>
      <div flex-grow h-0 md:h-full md:w-0 flex flex-col>
        <h2 md:text-xl pb-2 pl-4 select-none flex items-end>
          Output
        </h2>
        <div flex-grow relative max-h-full>
          <Editor v-model="result" lang="javascript" readonly class="w-full h-full max-h-full" />
          <!-- <div z-20 absolute left-1 right-2 bottom--2 children:p-2 children:px-3 children:b-2 children:rounded flex flex-col gap-2>
            <div v-if="error" relative text-red-200 bg-red-900 bg-op-80 b-red-500>
              <h3 text-lg pb-1>
                Error
              </h3>
              <div font-mono>
                {{ error }}
              </div>
              <button absolute right-3 top-3 w-6 h-6 b-none i-carbon-close @click="error = ''" />
            </div>
            <div v-if="loading" relative text-gray-300 bg-gray-900 bg-op-80 b-gray-500>
              <h3 text-lg pb-1>
                Running
              </h3>
              <div font-mono>
                Loading formatter
              </div>
              <button absolute right-3 top-3 w-6 h-6 b-none i-carbon-close @click="loading = false" />
            </div>
          </div> -->
        </div>
      </div>
    </div>
  </div>
</template>
