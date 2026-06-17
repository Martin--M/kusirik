import { createRouter, createWebHashHistory } from 'vue-router'
import { useProfileStore } from '@/stores/profile.store'

// Hash history is required in Tauri (no server to handle path-based routing)
const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
      meta: { requiresNoProfile: true },
    },
    {
      path: '/',
      redirect: '/live',
    },
    {
      path: '/live',
      name: 'live',
      component: () => import('@/views/LiveView.vue'),
      meta: { requiresProfile: true },
    },
    {
      path: '/movies',
      name: 'movies',
      component: () => import('@/views/MoviesView.vue'),
      meta: { requiresProfile: true },
    },
    {
      path: '/series',
      name: 'series',
      component: () => import('@/views/SeriesView.vue'),
      meta: { requiresProfile: true },
    },
    {
      path: '/search',
      name: 'search',
      component: () => import('@/views/SearchView.vue'),
      meta: { requiresProfile: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { requiresProfile: true },
    },
  ],
})

// Navigation guard: redirect to /setup if no profile exists
router.beforeEach(async (to) => {
  const profileStore = useProfileStore()

  // Load profile on first navigation if not yet loaded
  if (profileStore.profile === null && !profileStore.isLoading) {
    await profileStore.loadProfile()
  }

  const hasProfile = profileStore.hasProfile

  if (to.meta.requiresProfile && !hasProfile) {
    return { name: 'setup' }
  }

  if (to.meta.requiresNoProfile && hasProfile) {
    return { name: 'live' }
  }
})

export default router
