import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import { useI18n } from '@/composables/useI18n'
import './index.css'

const app = createApp(App)

// Expose lightweight translation and date formatter globally for Vue templates
const { t, formatTime } = useI18n()
app.config.globalProperties.$t = t
app.config.globalProperties.$formatTime = formatTime

app.use(createPinia())
app.use(router)
app.use(VueQueryPlugin, {
  queryClientConfig: {
    defaultOptions: {
      queries: {
        staleTime: Infinity,
        gcTime: 1000 * 60 * 30, // 30 mins
        retry: false,
        refetchOnWindowFocus: false,
      },
    },
  },
})

app.mount('#app')

