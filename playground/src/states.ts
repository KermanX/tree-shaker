import { compressToBase64, decompressFromBase64 } from 'lz-string'
import { tree_shake } from '@kermanx/tree-shaker'
import { computed, ref, watch, watchEffect } from 'vue'
import { DEMO } from './examples';

export const onInputUpdate: (()=>void)[] = []
export const input = ref('')
export const doTreeShake = ref(true)
export const doMinify = ref(false)

export const showLogs = ref(false)

export const debouncedInput = ref('')
let debounceTimeout = Number.NaN
watch(input, (input) => {
  clearInterval(debounceTimeout)
  debounceTimeout = setTimeout(() => {
    debouncedInput.value = input
  }, 300)
})

export function load(reset = false) {
  let parsed
  if (!reset && window.location.hash) {
    try {
      parsed = JSON.parse(decompressFromBase64(window.location.hash.slice(1)) || '{}')
    }
    catch (e) { console.error(e) }
  }
  parsed ||= {}
  debouncedInput.value = input.value = parsed.input ?? DEMO
  onInputUpdate.forEach(fn => fn())
  doTreeShake.value = parsed.doTreeShake ?? true
  doMinify.value = parsed.doMinify ?? false
  save()
}

function save() {
  window.location.hash = compressToBase64(JSON.stringify({
    input: input.value,
    doTreeShake: doTreeShake.value,
    doMinify: doMinify.value,
  }))
}

load()
watchEffect(save)

const result = computed(() => tree_shake(debouncedInput.value, doTreeShake.value, doMinify.value, true))
export const output = computed(() => result.value.output.trim() || `// Empty output or error`)
export const onlyMinifiedSize = computed(() => tree_shake(debouncedInput.value, false, true, false).output.length)
export const treeShakedUnminifiedSize = computed(() => doTreeShake.value && !doMinify.value ? result.value.output.length : tree_shake(debouncedInput.value, true, false, false).output.length)
export const treeShakedMinifiedSize = computed(() => doTreeShake.value && doMinify.value ? result.value.output.length : tree_shake(debouncedInput.value, true, true, false).output.length)
export const treeShakeRate = computed(() => 100 * treeShakedMinifiedSize.value / onlyMinifiedSize.value);
export const diagnostics = computed(() => {
  hideDiagnostics.value = false
  return result.value.diagnostics
})
export const hideDiagnostics = ref(false)
export const logsRaw = computed(() => result.value.logs
  // .filter(s =>
  //   !s.startsWith('PushExprSpan') &&
  //   !s.startsWith('PopExprSpan')
  // )
  .slice(0, 20000))

export const activeLogIndex = ref(5)
export const currentStmtSpan = ref<[number, number]>([0, 0])
export const currentExprSpan = ref<[number, number]>([0, 0])

export interface CallScope {
  span: [number, number]
  old_variable_scope_stack: number[]
  cf_scope_depth: number
  body_variable_scope: number
}
export const activeCallScope = ref(0);
export const currentCallScopes = ref<CallScope[]>([])

export interface CfScope {
  id: number
  kind: string
  exited: string
}
export const activeCfScope = ref(0)
export const currentCfScopes = ref<CfScope[]>([])

export interface VarScope {
  id: number
  cf_scope: number
}
export const activeVarScope = ref(0)
export const currentVarScopes = ref<VarScope[]>([])
export const currentVarStack = ref<number[]>([])

watchEffect(() => {
  const stmtSpans = []
  const exprSpans = []
  const callScopes: CallScope[] = [{
    span: [0, 0],
    old_variable_scope_stack: [],
    cf_scope_depth: 0,
    body_variable_scope: 0,
  }]
  const cfScopes: CfScope[] = [{
    id: 0,
    kind: 'Module',
    exited: 'false',
  }]
  const varScopes: VarScope[] = [{
    id: 0,
    cf_scope: 0,
  }]
  let varStack = [0]

  function parseSpan(span: string) {
    return span.split('-').map(Number) as [number, number]
  }

  for (const log of logsRaw.value.slice(0, activeLogIndex.value + 1)) {
    const [type, ...data] = log.split(' ')
    if (type === "PushStmtSpan") {
      stmtSpans.push(parseSpan(data[0]))
    } else if (type === "PopStmtSpan") {
      stmtSpans.pop()
    } else if (type === "PushExprSpan") {
      exprSpans.push(parseSpan(data[0]))
    } else if (type === "PopExprSpan") {
      exprSpans.pop()
    } else if (type === "PushCallScope") {
      const [span, old_variable_scope_stack, cf_scope_depth, body_variable_scope] = data
      callScopes.push({
        span: parseSpan(span),
        old_variable_scope_stack: old_variable_scope_stack.split(',').map(Number),
        cf_scope_depth: Number(cf_scope_depth),
        body_variable_scope: Number(body_variable_scope),
      })
    } else if (type === "PopCallScope") {
      callScopes.pop()
    } else if (type === "PushCfScope") {
      const [id, kind, exited] = data
      cfScopes.push({
        id: Number(id),
        kind,
        exited,
      })
    } else if (type === "UpdateCfScopeExited") {
      const [id, exited] = data
      cfScopes.find(scope => scope.id === Number(id))!.exited = exited
    } else if (type === "PopCfScope") {
      cfScopes.pop()
    } else if (type == "ReplaceVarScopeStack") {
      const [stack] = data
      varStack = stack.split(',').map(Number)
    } else if (type === "PushVarScope") {
      const [id, cf_scope] = data
      varStack.push(varScopes.length)
      varScopes.push({
        id: Number(id),
        cf_scope: Number(cf_scope),
      })
    } else if (type === "PopVarScope") {
      varStack.pop()
      varScopes.pop()
    }
  }

  currentStmtSpan.value = stmtSpans.at(-1) ?? [0, 0]
  currentExprSpan.value = exprSpans.at(-1) ?? [0, 0]
  currentCallScopes.value = callScopes.reverse()
  activeCallScope.value = 0
  currentCfScopes.value = cfScopes.reverse()
  activeCfScope.value = 0
  currentVarScopes.value = varScopes.reverse()
  currentVarStack.value = varStack
  activeVarScope.value = 0
})
