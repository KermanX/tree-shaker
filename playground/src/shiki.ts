import { shikiToMonaco } from '@shikijs/monaco'
import * as monaco from 'monaco-editor'
import { createHighlighter } from 'shiki'

let initialized = false

export async function setupShiki() {
  if (initialized)
    return
  initialized = true

  const highlighter = await createHighlighter({
    langs: ['javascript', 'rust'],
    themes: ['vitesse-dark', 'vitesse-light'],
  })

  shikiToMonaco(highlighter, monaco)
}
