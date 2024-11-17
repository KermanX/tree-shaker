<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watchEffect } from 'vue'
import * as monaco from 'monaco-editor'
import { onInputUpdate } from './states';

const props = defineProps<{
  lang: 'javascript' | 'rust' | 'markdown'
  readonly?: boolean,
  options?: Partial<monaco.editor.IStandaloneEditorConstructionOptions>
}>()

const emits = defineEmits<{
  'update:editor': [monaco.editor.IStandaloneCodeEditor]
}>()

const value = defineModel<string>({ required: true })

const container = ref<HTMLElement | null>(null)

onMounted(async () => {
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
    minimap: {
      enabled: false,
    },
    ...props.options,
  })

  emits('update:editor', editor)

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
  const index = onInputUpdate.length;
  onInputUpdate.push(async () => {
    await nextTick()
    editor.setValue(value.value)
  })
  onUnmounted(() => {
    onInputUpdate[index] = () => { }
  })
})
</script>

<template>
  <div ref="container" />
</template>
