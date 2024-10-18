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
})
