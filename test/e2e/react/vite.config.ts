import { defineConfig } from 'vite'
import React from '@vitejs/plugin-react'
import TreeShake from '../plugin'

const JSX_FN_RAW = `function q(c, a, g) {
  var b, d = {}, e = null, h = null;
  void 0 !== g && (e = "" + g);
  void 0 !== a.key && (e = "" + a.key);
  void 0 !== a.ref && (h = a.ref);
  for (b in a) m$1.call(a, b) && !p.hasOwnProperty(b) && (d[b] = a[b]);
  if (c && c.defaultProps) for (b in a = c.defaultProps, a) void 0 === d[b] && (d[b] = a[b]);
  return { $$typeof: k, type: c, key: e, ref: h, props: d, _owner: n.current };
}`

const JSX_FN_PLACEHOLDER = `JSX_FN_START();/* @__FINITE_RECURSION__ */function q(c, a, g) {
  return __JSX_BLACK_BOX__(typeof c === "string" ? [c,a] : c(a), g)
};JSX_FN_END();`

const JSX_FN_RESTORE_RE = /JSX_FN_START\(\);[\s\S]+JSX_FN_END\(\);/

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    React({
      // jsxRuntime: 'classic'
    }),
    TreeShake({
      pre: code => code.replace(JSX_FN_RAW, JSX_FN_PLACEHOLDER),
      post: code => code.replace(JSX_FN_RESTORE_RE, JSX_FN_RAW),
    }),
  ],
  define: {
    'process.env.NODE_ENV': '"production"',
  },
  build: {
    rollupOptions: {
      external: ['react', 'react-dom'],
    }
  }
})
