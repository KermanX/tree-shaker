import { compressToBase64, decompressFromBase64 } from 'lz-string'
import { tree_shake } from '@kermanx/tree-shaker'
import { computed, ref, shallowRef, toRef, watch, watchEffect } from 'vue'
import Editor from './Editor.vue'
import { DEMO } from './examples';
import Debugger from './Logs.vue';

export const input = ref('')
export const doTreeShake = ref(true)
export const doMinify = ref(false)

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

const result = computed(() => tree_shake(debouncedInput.value, doTreeShake.value, doMinify.value, true))
export const output = computed(() => result.value.output.trim() || `// Empty output or error`)
export const diagnostics = computed(() => result.value.diagnostics.join('\n'))
export const logsRaw = computed(() => result.value.logs)

export const activeLogIndex = ref(5)

export const activeStmtSpan = computed(() => {
  let t = 0;
  for (let i = activeLogIndex.value; i >= 0; i--) {
    if (logsRaw.value[i].startsWith('PopStmtSpan')) {
      t++;
      continue;
    }
    let match = logsRaw.value[i].match(/^PushStmtSpan (\d+)-(\d+)$/);
    if (match && (t-- === 0)) {
      return [Number(match[1]), Number(match[2])] as [number, number]
    }
  }
})

export const activeExprSpan = computed(() => {
  let t = 0;
  for (let i = activeLogIndex.value; i >= 0; i--) {
    if (logsRaw.value[i].startsWith('PopExprSpan')) {
      t++;
      continue;
    }
    let match = logsRaw.value[i].match(/^PushExprSpan (\d+)-(\d+)$/);
    if (match && (t-- === 0)) {
      return [Number(match[1]), Number(match[2])] as [number, number]
    }
  }
})
