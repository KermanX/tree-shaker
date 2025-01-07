import { defineConfig } from 'tsup'

export default defineConfig({
  entry: {
    bundled: './commonjs/main.js',
  },
  format: ['cjs'],
  target: 'node18',
  outDir: './commonjs/dist',
})
