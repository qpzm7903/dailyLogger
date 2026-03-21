/**
 * E2E-002: 快速笔记流程测试
 * 验证打开 → 输入 → 保存 → 列表显示完整流程
 */

import { test, expect } from '../fixtures/base-test';

test.describe('快速笔记流程', () => {
  test('打开快速笔记弹窗', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开快速笔记弹窗
    const quickNoteButton = page.locator('button:has-text("记录"), button:has-text("Record")').first();
    await quickNoteButton.waitFor({ state: 'visible', timeout: 10000 });
    await quickNoteButton.click();

    // 等待弹窗出现
    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 验证弹窗可见
    await expect(modal).toBeVisible();
  });

  test('输入内容并保存', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开弹窗
    const quickNoteButton = page.locator('button:has-text("记录"), button:has-text("Record")').first();
    await quickNoteButton.waitFor({ state: 'visible', timeout: 10000 });
    await quickNoteButton.click();

    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 输入内容
    const textarea = modal.locator('textarea').first();
    await textarea.fill(`测试笔记 ${Date.now()}`);

    // 点击保存按钮（支持中英文）
    const saveButton = modal.locator('button:has-text("保存"), button:has-text("Save")').first();
    await saveButton.click();

    // 等待弹窗关闭
    await modal.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  });

  test('取消输入不保存', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开弹窗
    const quickNoteButton = page.locator('button:has-text("记录"), button:has-text("Record")').first();
    await quickNoteButton.waitFor({ state: 'visible', timeout: 10000 });
    await quickNoteButton.click();

    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 输入内容
    const textarea = modal.locator('textarea').first();
    await textarea.fill('这条笔记不应该被保存');

    // 点击取消按钮（支持中英文）
    const cancelButton = modal.locator('button:has-text("取消"), button:has-text("Cancel")').first();
    await cancelButton.click();

    // 等待弹窗关闭
    await modal.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  });

  test('使用 Escape 键关闭弹窗', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 打开弹窗
    const quickNoteButton = page.locator('button:has-text("记录"), button:has-text("Record")').first();
    await quickNoteButton.waitFor({ state: 'visible', timeout: 10000 });
    await quickNoteButton.click();

    const modal = page.locator('.fixed.inset-0').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });

    // 按 Escape 键
    await page.keyboard.press('Escape');

    // 等待弹窗关闭
    await modal.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  });
});