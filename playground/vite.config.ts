import Unocss from 'unocss/vite'
import { defineConfig } from 'vite'
import Vue from '@vitejs/plugin-vue'
import Wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [Vue(), Unocss(), Wasm()],
  optimizeDeps: {
    exclude: [
      '@kermanx/tree-shaker',
    ],
    include: [
      'monaco-editor/esm/vs/editor/editor.worker',
      'monaco-editor/esm/vs/language/typescript/ts.worker',
    ],
  },
  build: {
    target: 'esnext',
  },
})
