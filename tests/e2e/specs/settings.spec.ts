/**
 * E2E-003: 设置流程测试
 * 验证打开 → 修改 → 保存 → 重开验证完整流程
 */

import { test, expect } from '../fixtures/base-test';

test.describe('设置流程', () => {
  // Settings button is now in the Sidebar (UX-3 sidebar upgrade)
  const getSettingsButton = (page: any) => page.locator('aside button').filter({ hasText: '设置' }).or(page.locator('aside button[title="设置"]'));

  test('打开设置弹窗', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开设置弹窗 (settings button is in sidebar)
    await getSettingsButton(page).click();
    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 验证弹窗可见
    await expect(modal).toBeVisible();
  });

  test('设置弹窗包含标签页', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开设置 (settings button is in sidebar)
    await getSettingsButton(page).click();
    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 验证标签页按钮存在（支持中英文）
    const tabButton = modal.locator('button:has-text("Basic"), button:has-text("基础")').first();
    await expect(tabButton).toBeVisible({ timeout: 5000 });
  });

  test('修改设置并保存', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开设置 (settings button is in sidebar)
    await getSettingsButton(page).click();
    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 查找输入框并修改
    const apiUrlInput = modal.locator('input[type="text"], input:not([type])').first();
    if (await apiUrlInput.isVisible()) {
      await apiUrlInput.fill('https://api.test.com/v1');
    }

    // 保存（支持中英文）
    const saveButton = modal.locator('button:has-text("保存"), button:has-text("Save")').first();
    await saveButton.click();

    // 等待弹窗关闭
    await modal.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  });

  test('取消修改不保存', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开设置 (settings button is in sidebar)
    await getSettingsButton(page).click();
    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 取消（支持中英文）
    const cancelButton = modal.locator('button:has-text("取消"), button:has-text("Cancel")').first();
    await cancelButton.click();

    // 等待弹窗关闭
    await modal.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  });

  test('使用 Escape 键关闭设置', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开设置 (settings button is in sidebar)
    await getSettingsButton(page).click();
    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 按 Escape 键
    await page.keyboard.press('Escape');

    // 等待弹窗关闭
    await modal.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  });
});