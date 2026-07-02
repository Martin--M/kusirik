import { ComponentCustomProperties } from 'vue'

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module '@vue/runtime-core' {
  interface ComponentCustomProperties {
    $t: (key: string, replacements?: Record<string, string | number | null | undefined>) => string
    $formatTime: (isoString: string | null) => string
  }
}
