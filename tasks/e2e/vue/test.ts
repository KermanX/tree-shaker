/// <reference types="@vitest/browser/matchers" />
import { expect, test } from 'vitest'
import { page } from '@vitest/browser/context'

test('harness', async () => {
  document.body.innerHTML = `<div id="app"></div>`

  await import('./dist/shaken.js')

  await expect.element(page.getByText('Hello _Kerman')).toBeInTheDocument()

  const counter = page.getByRole('button')
  await counter.click()
  await expect.element(counter).toHaveTextContent('1')
  await counter.click()
  await expect.element(counter).toHaveTextContent('2')
  await counter.click()
  await expect.element(counter).toHaveTextContent('3')

  const input = page.getByRole('textbox')
  await input.fill('Oxc')
  await expect.element(page.getByText('Hello Oxc')).toBeInTheDocument()
})
