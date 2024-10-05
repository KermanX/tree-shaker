import { shikiToMonaco } from '@shikijs/monaco'
import * as monaco from 'monaco-editor'
import { createHighlighter } from 'shiki'

const highlighter = await createHighlighter({
  langs: ['javascript', 'rust', 'markdown'],
  themes: ['vitesse-dark', 'vitesse-light'],
})

shikiToMonaco(highlighter, monaco)
