/// <reference types="vite/client" />

import App from './App.vue'
import { createApp } from 'vue'
import { createVuetify } from 'vuetify'

createApp(App)
  .use(createVuetify())
  .mount('#app')
