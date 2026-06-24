<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ImageDataResponse {
  bytes: number[]
  mime: string
}

const props = withDefaults(
  defineProps<{
    src?: string | null
    alt?: string
    fallbackText?: string
  }>(),
  {
    src: null,
    alt: 'Image',
    fallbackText: ''
  }
)

const cachedSrc = ref<string | null>(null)
const isLoading = ref(false)
const hasError = ref(false)

// Generate a deterministic gradient class based on hashing the fallbackText
const fallbackBgClass = ref('fallback-bg-0')

function generateFallbackClass(text: string) {
  if (!text) return 'fallback-bg-0'
  let hash = 0
  for (let i = 0; i < text.length; i++) {
    hash = text.charCodeAt(i) + ((hash << 5) - hash)
  }
  const index = Math.abs(hash) % 5 // 5 different gradient variations
  return `fallback-bg-${index}`
}

const initials = ref('')

function generateInitials(text: string) {
  if (!text) return '?'
  const cleanText = text.replace(/[^a-zA-Z0-9\s]/g, '').trim()
  const words = cleanText.split(/\s+/)
  if (words.length >= 2) {
    return (words[0][0] + words[1][0]).toUpperCase()
  }
  return cleanText.slice(0, 2).toUpperCase()
}

function revokeCurrentUrl() {
  if (cachedSrc.value) {
    URL.revokeObjectURL(cachedSrc.value)
    cachedSrc.value = null
  }
}

async function loadImage() {
  if (!props.src) {
    hasError.value = true
    return
  }

  isLoading.value = true
  hasError.value = false
  revokeCurrentUrl()

  try {
    const res = await invoke<ImageDataResponse>('get_image_data', { url: props.src })
    if (res && res.bytes) {
      const blob = new Blob([new Uint8Array(res.bytes)], { type: res.mime })
      cachedSrc.value = URL.createObjectURL(blob)
    } else {
      hasError.value = true
    }
  } catch (e) {
    console.error(`Failed to load cached image: ${props.src}`, e)
    hasError.value = true
  } finally {
    isLoading.value = false
  }
}

watch(
  () => props.src,
  () => {
    revokeCurrentUrl()
    loadImage()
  }
)

watch(
  () => props.fallbackText,
  (newText) => {
    fallbackBgClass.value = generateFallbackClass(newText)
    initials.value = generateInitials(newText)
  },
  { immediate: true }
)

onMounted(() => {
  loadImage()
})

onBeforeUnmount(() => {
  revokeCurrentUrl()
})
</script>

<template>
  <div class="cached-image-container">
    <div v-if="isLoading" class="skeleton-loader"></div>
    
    <img
      v-else-if="!hasError && cachedSrc"
      :src="cachedSrc"
      :alt="alt"
      class="loaded-image"
      @error="hasError = true"
    />
    
    <div v-else class="fallback-placeholder" :class="fallbackBgClass">
      <span class="placeholder-text">{{ initials }}</span>
    </div>
  </div>
</template>

<style scoped>
.cached-image-container {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: inherit;
  overflow: hidden;
  background-color: rgba(255, 255, 255, 0.05);
}

.loaded-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.skeleton-loader {
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, rgba(255,255,255,0.03) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.03) 75%);
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
}

@keyframes loading {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.fallback-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-weight: 700;
  font-size: 0.9rem;
  letter-spacing: -0.025em;
  user-select: none;
}

/* Premium Gradient Variations */
.fallback-bg-0 { background: linear-gradient(135deg, #ef4444, #f97316); }
.fallback-bg-1 { background: linear-gradient(135deg, #3b82f6, #8b5cf6); }
.fallback-bg-2 { background: linear-gradient(135deg, #10b981, #06b6d4); }
.fallback-bg-3 { background: linear-gradient(135deg, #f59e0b, #eab308); }
.fallback-bg-4 { background: linear-gradient(135deg, #ec4899, #f43f5e); }

.placeholder-text {
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}
</style>
