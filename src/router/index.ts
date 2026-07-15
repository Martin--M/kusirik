import { createRouter, createWebHashHistory } from 'vue-router'
import { useProfileStore } from '@/stores/profile.store'
import { getSyncStatus } from '@/lib/tauri-commands'
import GuideView from '@/views/GuideView.vue'

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
      path: '/guide',
      name: 'guide',
      component: GuideView,
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
      path: '/favorites',
      name: 'favorites',
      component: () => import('@/views/FavoritesView.vue'),
      meta: { requiresProfile: true },
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('@/views/HistoryView.vue'),
      meta: { requiresProfile: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { requiresProfile: false },
    },
  ],
})

let cachedSyncComplete = false

// Navigation guard: redirect to /setup if no profile exists or initial sync is incomplete
router.beforeEach(async (to) => {
  const profileStore = useProfileStore()

  // Load profiles on first navigation if not yet loaded
  if (profileStore.profiles.length === 0 && !profileStore.isLoading) {
    await profileStore.loadProfiles()
  }

  const hasProfile = profileStore.hasProfile
  if (!hasProfile) {
    cachedSyncComplete = false;
  }

  let isSyncComplete = cachedSyncComplete
  if (hasProfile && !isSyncComplete) {
    try {
      for (const profile of profileStore.profiles) {
        const statusList = await getSyncStatus(profile.id)
        const live = statusList.find((s) => s.data_type === 'live_streams')
        const vod = statusList.find((s) => s.data_type === 'vod_streams')
        const series = statusList.find((s) => s.data_type === 'series')

        if (live?.fetched_at && vod?.fetched_at && series?.fetched_at) {
          isSyncComplete = true
          cachedSyncComplete = true
          break
        }
      }
    } catch (e) {
      console.error('Failed to get sync status:', e)
    }
  }

  if (to.meta.requiresProfile) {
    if (!hasProfile) {
      return { name: 'setup' }
    }
    if (!isSyncComplete && to.name !== 'setup') {
      return { name: 'setup' }
    }
  }

  if (to.meta.requiresNoProfile && hasProfile && isSyncComplete && !to.query.add && !to.query.id) {
    return { name: 'live' }
  }
})

export default router
