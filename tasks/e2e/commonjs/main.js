import { f } from "./mod.cjs";
import { g } from "./mod2.js";

export function main() {
  console.log(f(1), g(true));
}
