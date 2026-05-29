import { createApp } from 'vue'
import App from './App.vue'
import { createAppProviders } from './app/providers'
import './app/styles/index.scss'

function escapeHtml(value: string) {
  return value.replace(/[&<>]/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[char] ?? char)
}

function showFatalError(error: unknown) {
  const message = error instanceof Error ? error.stack || error.message : String(error)
  const target = document.querySelector('#app')
  if (!target) return
  target.innerHTML = `<div style="min-height:100vh;display:flex;align-items:center;justify-content:center;padding:32px;background:var(--vt-bootstrap-bg, #fafafa);color:var(--vt-bootstrap-fg, #52525b);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;"><pre style="max-width:960px;white-space:pre-wrap;border:1px solid rgba(127,127,127,.2);border-radius:12px;background:transparent;padding:18px;">${escapeHtml(message)}</pre></div>`
}

window.addEventListener('error', (event) => showFatalError(event.error ?? event.message))
window.addEventListener('unhandledrejection', (event) => showFatalError(event.reason))

try {
  const app = createApp(App)
  app.config.errorHandler = (error) => showFatalError(error)
  createAppProviders(app)
  app.mount('#app')
} catch (error) {
  showFatalError(error)
}
