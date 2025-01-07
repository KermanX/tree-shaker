/// <reference types="vite/client" />
import { createApp } from 'vue'
import './monaco.ts'
import './styles.css'
import App from './App.vue'
import 'uno.css'
import '@unocss/reset/tailwind.css'
import './shiki.js'

createApp(App).mount('#app')
