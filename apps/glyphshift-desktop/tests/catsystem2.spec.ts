import { expect, test } from '@playwright/test'
import { readFileSync } from 'node:fs'
import { model, storageKey, workflowEditor } from './fixtures/productModel'

test('CatSystem2 catalog entry can be selected for text replacement without advertising font support', async ({ page }) => {
  const presentations = JSON.parse(readFileSync(new URL('../../../scripts/runtime-bundle-adapters.zh-CN.json', import.meta.url), 'utf8'))
  const presentation = presentations.find((entry: { id: string }) => entry.id === 'windows.catsystem2.utf8-text')
  const snapshot = structuredClone(model)
  snapshot.adapters.push({
    ...presentation, version: '1.0.0', platforms: ['windows'], technologies: [presentation.technology],
    features: ['textObserve', 'textReplace'], configuration: 'none',
  })
  await page.addInitScript(({ key, value }) => localStorage.setItem(key, JSON.stringify(value)), { key: storageKey, value: snapshot })
  await page.goto('/')
  await page.getByRole('button', { name: '编辑 默认创作工作流' }).click()
  const editor = workflowEditor(page)
  await editor.getByRole('tab', { name: '软件与兼容方式' }).click()
  const row = editor.getByTestId('workflow-adapter-table').locator('tbody > tr').filter({ hasText: presentation.name })
  await expect(row).toBeVisible()
  await expect(row).toContainText('CatSystem2')
  await row.getByRole('checkbox').check()
  await expect(row.getByRole('checkbox')).toBeChecked()
  await expect(row).not.toContainText('字体替换')
})
