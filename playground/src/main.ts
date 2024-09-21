/// <reference types="vite/client" />
import { createApp } from 'vue'
import './monaco.ts'
import './styles.css'
import App from './App.vue'
import 'uno.css'
import '@unocss/reset/tailwind.css'

createApp(App).mount('#app')
