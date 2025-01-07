import Vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import TreeShake from '../plugin'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    Vue(),
    TreeShake(),
  ],
  define: {
    'process.env.NODE_ENV': '"production"'
  },
})
