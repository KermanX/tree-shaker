import { createLogger, Plugin } from "vite";
import { treeShake } from "@kermanx/tree-shaker"
import pico from "picocolors";

export default function (): Plugin | false {
  const logger = createLogger("info", {
    prefix: "tree-shaker"
  })

  const disabled = +(process.env.DISABLE_TREE_SHAKE ?? 0);

  return {
    name: "tree-shaker",
    enforce: 'post',
    apply: 'build',
    config(config) {
      return {
        build: {
          // Currently enabling Rollup treeshake because JS built-ins is not supported yet
          // rollupOptions: {
          //   treeshake: false
          // },
          outDir: './dist',
          minify: false,
          emptyOutDir: false,
          ...config?.build,
          lib: {
            entry: './main.ts',
            formats: ['es'],
            fileName: disabled ? 'bundled' : 'shaken',
            ...config?.build?.lib,
          },
        }
      }
    },
    renderChunk: {
      order: 'post',
      handler(code) {
        if (disabled) {
          return code;
        }
        const startTime = Date.now();
        const { output, diagnostics } = treeShake(code, "recommended", false);
        const duration = `${Date.now() - startTime}ms`;
        logger.info(pico.yellowBright(`\ntree-shake duration: ${duration}`));
        for (const diagnostic of diagnostics) {
          logger.error(diagnostic);
        }
        return output;
      }
    }
  }
}
