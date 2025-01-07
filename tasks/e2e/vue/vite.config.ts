import { defineConfig } from 'vite'
import Vue from '@vitejs/plugin-vue'
import TreeShake from '../plugin'

export default defineConfig({
  plugins: [
    Vue(),
    TreeShake(),
  ],
  define: {
    'process.env.NODE_ENV': '"production"'
  },
})
