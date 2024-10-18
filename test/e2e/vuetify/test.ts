/// <reference types="@vitest/browser/matchers" />
import { expect, test } from 'vitest'
import { page } from '@vitest/browser/context'

test('harness', async () => {
  const root = document.createElement('div')
  root.id = 'app'
  document.body.append(root)

  await import('./dist/shaken.js')

  await expect.element(page.getByText('Vuetify')).toBeInTheDocument()
})
