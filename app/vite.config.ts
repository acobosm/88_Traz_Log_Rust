import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { nodePolyfills } from 'vite-plugin-node-polyfills'

export default defineConfig({
  plugins: [
    vue(),
    nodePolyfills({ include: ['buffer', 'crypto', 'stream', 'process'] }),
  ],
  define: {
    'process.env.ANCHOR_BROWSER': 'true',
  },
})
