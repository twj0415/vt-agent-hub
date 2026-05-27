import { createApp } from 'vue'
import App from './App.vue'
import { createAppProviders } from './app/providers'
import './app/styles/index.scss'

const app = createApp(App)

createAppProviders(app)

app.mount('#app')
