import { createLogger, Plugin } from "vite";
import pico from "picocolors";

export default function (options: {
  pre?: (code: string) => string,
  post?: (code: string) => string,
} = {}): Plugin | false {
  const logger = createLogger("info", {
    prefix: "tree-shaker"
  })

  const disabled = +(process.env.DISABLE_TREE_SHAKE ?? 0);
  const treeShake = disabled ? null : import("@kermanx/tree-shaker");

  return {
    name: "tree-shaker",
    enforce: 'post',
    apply: 'build',
    config(config) {
      return {
        build: {
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
          rollupOptions: {
            ...config?.build?.rollupOptions,
            treeshake: false,
          },
          modulePreload: {
            polyfill: false,
            ...(typeof config?.build?.modulePreload === 'object' ? config.build.modulePreload : {}),
          }
        }
      }
    },
    renderChunk: {
      order: 'post',
      async handler(code) {
        if (disabled) {
          return code;
        }
        code = options.pre?.(code) ?? code;
        const startTime = Date.now();
        const { output, diagnostics } = (await treeShake).treeShake(code, "recommended", false);
        const duration = `${Date.now() - startTime}ms`;
        logger.info(pico.yellowBright(`\ntree-shake duration: ${duration}`));
        for (const diagnostic of diagnostics) {
          logger.error(diagnostic);
        }
        return options.post?.(output) ?? output;
      }
    }
  }
}
