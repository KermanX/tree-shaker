<script setup lang="ts">
import { onMounted, ref, watchEffect } from 'vue'
import * as monaco from 'monaco-editor'
import { setupShiki } from './shiki'

const props = defineProps<{
  lang: 'javascript' | 'rust'
  readonly?: boolean
}>()

const value = defineModel<string>({ required: true })

const container = ref<HTMLElement | null>(null)

onMounted(async () => {
  await setupShiki()

  const editor = monaco.editor.create(container.value!, {
    value: value.value,
    language: props.lang,
    readOnly: props.readonly,
    automaticLayout: true,
    lineNumbersMinChars: 3,
    wordWrap: 'on',
    wordWrapColumn: 80,
    padding: {
      top: 16,
    },
    tabSize: 2,
  })

  if (props.readonly) {
    watchEffect(() => {
      editor.setValue(value.value)
    })
  }
  else {
    editor.onDidChangeModelContent(() => {
      value.value = editor.getValue()
    })
  }
})
</script>

<template>
  <div ref="container" />
</template>
