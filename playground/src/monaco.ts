import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker.js?worker'
import TsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker'
import * as monaco from 'monaco-editor'

window.MonacoEnvironment = {
  getWorker(_, label) {
    if (label === 'typescript' || label === 'javascript')
      return new TsWorker()
    return new EditorWorker()
  },
}

monaco.languages.register({ id: 'javascript' })
monaco.languages.register({ id: 'typescript' })
