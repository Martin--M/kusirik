<script setup lang="ts">
import { useToastStore } from '@/stores/toast.store'
import IconSuccess from '../icons/IconSuccess.vue'
import IconError from '../icons/IconError.vue'

const toastStore = useToastStore()
</script>

<template>
  <transition name="toast-fade">
    <div v-if="toastStore.show" class="toast-message" :class="toastStore.type">
      <IconSuccess v-if="toastStore.type === 'success'" class="toast-icon" />
      <IconError v-else class="toast-icon" />
      <span>{{ toastStore.message }}</span>
    </div>
  </transition>
</template>

<style scoped>
.toast-message {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: 12px 20px;
  border-radius: var(--radius-md);
  color: #fff;
  font-weight: 600;
  font-size: 0.9rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 2000;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.toast-message.success {
  background-color: rgba(34, 197, 94, 0.9);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.toast-message.error {
  background-color: rgba(239, 68, 68, 0.9);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.toast-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.25s ease-out;
}

.toast-fade-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

@media (max-width: 768px) {
  .toast-message {
    left: 50%;
    right: auto;
    transform: translateX(-50%);
    bottom: 80px;
  }

  .toast-fade-enter-from {
    transform: translate(-50%, 20px);
  }
  .toast-fade-leave-to {
    transform: translate(-50%, -20px);
  }
}
</style>
