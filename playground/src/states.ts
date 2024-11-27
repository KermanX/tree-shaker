import { compressToBase64, decompressFromBase64 } from 'lz-string'
import { tree_shake } from '@kermanx/tree-shaker'
import { computed, ref, watch, watchEffect } from 'vue'
import { DEMO } from './examples';

export const onInputUpdate: (() => void)[] = []
export const input = ref('')
export const preset = ref('recommended')
export const doMinify = ref(false)
export const alwaysInline = ref(false)

export const debouncedInput = ref('')
let debounceTimeout = Number.NaN
watch(input, (input) => {
  clearInterval(debounceTimeout)
  debounceTimeout = setTimeout(() => {
    debouncedInput.value = input
  }, 300)
})

export function load(reset = false) {
  let parsed
  if (!reset && window.location.hash) {
    try {
      parsed = JSON.parse(decompressFromBase64(window.location.hash.slice(1)) || '{}')
    }
    catch (e) { console.error(e) }
  }
  parsed ||= {}
  debouncedInput.value = input.value = parsed.input ?? DEMO
  onInputUpdate.forEach(fn => fn())
  preset.value = parsed.preset ?? (parsed.doTreeShake != null ? (parsed.doTreeShake ? 'recommended' : 'disabled') : 'recommended')
  doMinify.value = parsed.doMinify ?? false
  alwaysInline.value = parsed.alwaysInline ?? false
  save()
}

function save() {
  window.location.hash = compressToBase64(JSON.stringify({
    input: input.value,
    preset: preset.value,
    doMinify: doMinify.value,
    alwaysInline: alwaysInline.value,
  }))
}

load()
watchEffect(save)

const minifiedOnly = computed(() => tree_shake(debouncedInput.value, "disabled", true, false))
const treeShakedOnly = computed(() => tree_shake(debouncedInput.value, preset.value, false, false))
const treeShakedMinified = computed(() => tree_shake(treeShakedOnly.value.output, "disabled", true, false))

const result = computed(() => {
  return {
    diagnostics: treeShakedOnly.value.diagnostics,
    output: doMinify.value ? treeShakedMinified.value.output : treeShakedOnly.value.output,
  }
})
export const output = computed(() => result.value.output.trim() || `// Empty output or error`)
export const onlyMinifiedSize = computed(() => minifiedOnly.value.output.length)
export const treeShakedUnminifiedSize = computed(() => treeShakedOnly.value.output.length)
export const treeShakedMinifiedSize = computed(() => treeShakedMinified.value.output.length)
export const treeShakeRate = computed(() => 100 * treeShakedMinifiedSize.value / onlyMinifiedSize.value);
export const diagnostics = computed(() => {
  hideDiagnostics.value = false
  return result.value.diagnostics
})
export const hideDiagnostics = ref(false)
