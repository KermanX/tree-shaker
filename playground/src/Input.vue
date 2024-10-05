<script setup lang="ts">
import * as monaco from 'monaco-editor';
import { watchEffect } from 'vue';
import Editor from './Editor.vue';
import { activeCallScope, currentCallScopes, currentExprSpan, currentStmtSpan, input } from './states';

function setupEditor(editor: monaco.editor.IStandaloneCodeEditor) {
  const model = editor.getModel()!;

  const activeStmtSpanDeco = editor.createDecorationsCollection();
  const activeExprSpanDeco = editor.createDecorationsCollection();
  const activeCallScopeDeco = editor.createDecorationsCollection();

  watchEffect(() => {
    let stmtSpan = currentStmtSpan.value;
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

    editor.revealLineInCenter(model.getPositionAt(stmtSpan[0]).lineNumber, monaco.editor.ScrollType.Smooth);

    const exprSpan = currentExprSpan.value;
    activeExprSpanDeco.set([{
      range: exprSpan ? monaco.Range.fromPositions(
        model.getPositionAt(exprSpan[0]),
        model.getPositionAt(exprSpan[1])
      ) : new monaco.Range(0, 0, 0, 0),
      options: {
        className: "active-expr-span",
      },
    }])

    const callSpan = currentCallScopes.value[activeCallScope.value].span;
    activeCallScopeDeco.set([{
      range: callSpan ? monaco.Range.fromPositions(
        model.getPositionAt(callSpan[0]),
        model.getPositionAt(callSpan[1])
      ) : new monaco.Range(0, 0, 0, 0),
      options: {
        blockClassName: "active-call-scope",
        blockPadding: [0, 0, 0, 2],
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
