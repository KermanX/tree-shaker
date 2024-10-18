import { createLogger, Plugin } from "vite";
import { treeShake } from "@kermanx/tree-shaker"
import { yellowBright } from "picocolors";

export default function (): Plugin {
  const logger = createLogger("info", {
    prefix: "tree-shaker"
  })

  return {
    name: "tree-shaker",
    enforce: 'post',
    apply: 'build',
    renderChunk: {
      order: 'post',
      handler(code) {
        const startTime = Date.now();
        const { output, diagnostics } = treeShake(code, "recommended", false, false);
        const duration = `${Date.now() - startTime}ms`;
        logger.info(yellowBright(`\ntree-shake duration: ${duration}`));
        for (const diagnostic of diagnostics) {
          logger.error(diagnostic);
        }
        return output;
      }
    }
  }
}
