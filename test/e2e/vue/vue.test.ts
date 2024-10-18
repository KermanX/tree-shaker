/// <reference types="@vitest/browser/matchers" />
import { expect, test } from 'vitest'
import { page } from '@vitest/browser/context'

test('harness', async () => {
  const root = document.createElement('div')
  root.id = 'app'
  document.body.append(root)

  await import('./dist/out.mjs')

  await expect.element(page.getByText('Hello World')).toBeInTheDocument()

  const counter = page.getByRole('button')
  await counter.click()
  await expect.element(counter).toHaveTextContent('1')
  await counter.click()
  await expect.element(counter).toHaveTextContent('2')
  await counter.click()
  await expect.element(counter).toHaveTextContent('3')
})
