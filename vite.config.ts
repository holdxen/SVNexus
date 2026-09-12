import process from 'node:process'

import semiTheming from '@douyinfe/semi-vite-plugin'
import react from '@vitejs/plugin-react'
import wyw from '@wyw-in-js/vite'
import { defineConfig } from 'vite'
import svgr from 'vite-plugin-svgr'

import { injectSourceLocation } from './vite-plugins/inject-source-location.ts'
const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig(() => ({
  build: {
    sourcemap: true,
  },
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    injectSourceLocation(),
    semiTheming({
      variables: {
        '$font-family-regular': 'JetBrains Mono, sans-serif',
        '$spacing-descriptions_item-paddingBottom': '0px',
        '$spacing-card-margin': '0px',
        '$spacing-card-padding': '0px',
        '$animation_duration-modal-hide': '200ms',
        '$animation_duration-modal-show': '200ms',
        '$spacing-modal_footer-marginY': '0px',
        '$spacing-descriptions_th-paddingRight': '0px',
        '$width-horizontal-handler': '5px',
        '$spacing-list_item-paddingX': '0px',
        '$spacing-list_item-paddingY': '0px',
        '$spacing-list_footer-paddingX': '0px',
        '$spacing-list_footer-paddingY': '0px',
        '$spacing-tabs_bar_line_tab-marginRight': '0px',
        '$color-card_body-text': 'var(--semi-color-text-0)',
        '$width-spin_small': '16px',
        '$spacing-input-paddingRight': '2px',
        '$spacing-input-paddingLeft': '2px',
      },
    }),
    wyw({
      include: ['**/*.{ts,tsx,js,jsx}'],
      exclude: ['**/node_modules/**'],

      // 让 WyW 正确解析 TSX 和 React 自动 JSX runtime
      oxcOptions: {
        transform: {
          jsx: {
            runtime: 'automatic',
          },
        },
      },
    }),
    react({
      compiler: {
        target: '19',
      },
    }),
    svgr(),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}))
