import { createApp } from 'vue'
import App from './App.vue'
import './styles/shared.css'

const app = createApp(App)

app.config.errorHandler = (err, instance, info) => {
  console.error('[Global Error]', err, '\nComponent:', instance, '\nInfo:', info)
}

app.mount('#app')
