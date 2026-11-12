import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './styles/app.css'
import App from './App.vue'
// Base Vue Flow styles are required: without them nodes are not absolutely
// positioned, so they stack in document flow and get clipped by the canvas.
import '@vue-flow/core/dist/style.css'

const app = createApp(App)
app.use(createPinia())
app.mount('#app')
