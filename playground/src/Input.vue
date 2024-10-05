<script setup lang="ts">
import * as monaco from 'monaco-editor';
import { watchEffect } from 'vue';
import Editor from './Editor.vue';
import { activeExprSpan, activeStmtSpan, input } from './states';

function setupEditor(editor: monaco.editor.IStandaloneCodeEditor) {
  const model = editor.getModel()!;

  const activeStmtSpanDeco = editor.createDecorationsCollection();
  const activeExprSpanDeco = editor.createDecorationsCollection();
  watchEffect(() => {
    let stmtSpan = activeStmtSpan.value;
    activeStmtSpanDeco.set([{
      range: stmtSpan ? monaco.Range.fromPositions(
        model.getPositionAt(stmtSpan[0]),
        model.getPositionAt(stmtSpan[1])
      ) : new monaco.Range(0, 0, 0, 0),
      options: {
        className: "active-stmt-span",
        glyphMarginClassName: "active-stmt-indicator",
      },
    }])

    const exprSpan = activeExprSpan.value;
    activeExprSpanDeco.set([{
      range: exprSpan ? monaco.Range.fromPositions(
        model.getPositionAt(exprSpan[0]),
        model.getPositionAt(exprSpan[1])
      ) : new monaco.Range(0, 0, 0, 0),
      options: {
        className: "active-expr-span",
      },
    }])
  })
}
</script>

<template>
  <Editor ref="editorComp" v-model="input" lang="javascript" @update:editor="setupEditor" :options="{
    glyphMargin: true,
  }" />
</template>
