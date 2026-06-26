# UI Design System & Component Guidelines

This document details the visual guidelines, design tokens, and reusable component APIs that establish the Kusirik interface.

---

## 🎨 Design Tokens

All core styles are centralized in [index.css](../src/index.css) as CSS custom properties.

### Colors
| Token Name | Light Theme | Dark Theme | Purpose |
| :--- | :--- | :--- | :--- |
| `--color-bg` | `#f9f9f9` | `#0f172a` | Root background color. |
| `--color-surface` | `#ffffff` | `#1e293b` | Panels, cards, and dropdown containers. |
| `--color-text` | `#111827` | `#f8fafc` | Primary typography. |
| `--color-text-muted` | `#6b7280` | `#94a3b8` | Subtitles, labels, and secondary details. |
| `--color-primary` | `#3b82f6` | `#60a5fa` | Main buttons, active sidebar items. |
| `--color-primary-hover`| `#2563eb` | `#3b82f6` | Interactive hover indicator. |
| `--color-border` | `#e5e7eb` | `#334155` | Borders, divider lines. |

### Spacing & Layout
* **Spacing Scale**: Rem-based multiples (`--spacing-1: 0.25rem`, `--spacing-2: 0.5rem`, `--spacing-3: 0.75rem`, `--spacing-4: 1rem`, `--spacing-6: 1.5rem`, `--spacing-8: 2rem`).
* **Border Radius**: `--radius-sm` (4px), `--radius-md` (6px), `--radius-lg` (8px).
* **Transitions**: `--transition-fast` (150ms), `--transition-normal` (250ms) using `ease-in-out` curves.

---

## 🔍 Glassmorphism & Aesthetics

Kusirik uses a modern dark-mode-first aesthetic with "glass" elements:
- **Card Styling**: Semi-transparent background overlays on dark surfaces combined with a subtle border (`--color-border`).
- **Typography**: Uses clean `Inter` system sans-serif fonts. Headings feature a `font-weight: 600`.
- **Micro-Animations**: Hover animations on stream card lists scale slightly and raise shadows to create tactile depth.

---

## 🧩 Reusable Component Catalog

To keep routed view sizes compact, layout code is modularized into reusable components located under `src/components/ui/`:

### 1. Custom Select ([CustomSelect.vue](../src/components/ui/CustomSelect.vue))
Re-implements native dropdown controls with a glassmorphic select element.
- **Features**: Close-on-click-away behavior, slot support, and keyboard escape bindings.

### 2. Category Sidebar ([CategorySidebar.vue](../src/components/ui/CategorySidebar.vue))
Displays stream categories. Renders responsively depending on screen constraints:
- **Desktop**: Vertical sidebar navigation.
- **Mobile**: Horizontal scrollable chips array.

### 3. Filter Header ([FilterHeader.vue](../src/components/ui/FilterHeader.vue))
Consolidates layout actions for streams:
- Renders search input elements.
- Hosts sorting actions, theme triggers, and layout grid density controls.

### 4. Detail Panel ([StreamDetailPanel.vue](../src/components/ui/StreamDetailPanel.vue))
Combines the media detail layout (play/copy actions, metadata display, images loaders):
- **Desktop**: Sliding side panel overlay.
- **Mobile**: Swipeable bottom sheet.

### 5. Application Toasts ([AppToast.vue](../src/components/ui/AppToast.vue))
Mounted at the root level in `App.vue`. Listens to the global `useToastStore` to display smooth slide-in notifications with a auto-timeout of 3 seconds.

### 6. Cached Image ([CachedImage.vue](../src/components/ui/CachedImage.vue))
Wraps image loading with placeholder fallbacks and offline BLOB cache lookup capabilities, powered by `IntersectionObserver` for lazy loading.

### 7. List Row Item ([ListRowItem.vue](../src/components/ui/ListRowItem.vue))
A unified list row component shared between movie and series listing layouts.
- **Features**: Structured layout with a cover image, title, rating with custom inline SVG star (visible only when rated > 0), release year, selection highlights, and hover transitions.

