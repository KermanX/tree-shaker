import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import treeShake from '../plugin'

export default defineConfig({
  plugins: [
    vue({
      isProduction: true,
    }),
    treeShake(),
  ],
  define: {
    'process.env.NODE_ENV': '"production"'
  },
  build: {
    lib: {
      entry: './index.ts',
      formats: ['es'],
      fileName: 'out'
    },
    outDir: './dist',
    minify: false,
  }
})
