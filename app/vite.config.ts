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
  server: {
    headers: {
      // @solana/web3.js y @coral-xyz/anchor usan eval() internamente
      'Content-Security-Policy': "script-src 'self' 'unsafe-eval' 'unsafe-inline'; worker-src blob:;",
    },
  },
})
