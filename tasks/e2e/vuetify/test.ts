/// <reference types="@vitest/browser/matchers" />
import { expect, test } from 'vitest'
import { page } from '@vitest/browser/context'

test('harness', async () => {
  document.body.innerHTML = `<div id="app"></div>`

  await import('./dist/shaken.js')

  await expect.element(page.getByText('Welcome')).toBeInTheDocument()
})
