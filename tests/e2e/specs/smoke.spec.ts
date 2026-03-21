/**
 * E2E-001: 冒烟测试
 * 验证应用能正确加载，header 和主要组件可见
 */

import { test, expect } from '../fixtures/base-test';

test.describe('冒烟测试', () => {
  test('应用加载完成，header 可见', async ({ page }) => {
    // 导航到主页面
    await page.goto('/');

    // 等待 header 加载
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 验证 header 存在
    await expect(page.locator('header')).toBeVisible();

    // 验证标题
    await expect(page.locator('h1:has-text("DailyLogger")')).toBeVisible();
  });

  test('快速记录按钮存在', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 验证快速记录按钮存在（支持中英文）
    const quickNoteButton = page.locator('button:has-text("记录"), button:has-text("Record")').first();
    await expect(quickNoteButton).toBeVisible({ timeout: 10000 });
  });

  test('设置按钮存在', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 验证设置按钮存在
    const settingsButton = page.locator('button:has-text("⚙️")');
    await expect(settingsButton).toBeVisible({ timeout: 10000 });
  });

  test('今日工作流区域显示', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 等待主要内容加载 - 使用更通用的选择器
    await page.locator('h2').first().waitFor({ state: 'visible', timeout: 10000 });

    // 验证今日工作流区域存在
    await expect(page.locator('h2').first()).toBeVisible();
  });

  test('快速记录按钮可点击', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 等待快速记录按钮可见
    const quickNoteButton = page.locator('button:has-text("记录"), button:has-text("Record")').first();
    await quickNoteButton.waitFor({ state: 'visible', timeout: 10000 });

    // 点击快速记录按钮
    await quickNoteButton.click();

    // 验证弹窗出现 - 等待弹窗容器
    const modal = page.locator('.fixed.inset-0, [class*="modal"]').first();
    await expect(modal).toBeVisible({ timeout: 5000 });
  });

  test('设置按钮可点击', async ({ page }) => {
    await page.goto('/');
    await page.locator('header').waitFor({ state: 'visible', timeout: 15000 });

    // 点击设置按钮
    await page.locator('button:has-text("⚙️")').click();

    // 验证设置弹窗出现
    const modal = page.locator('.fixed.inset-0, [class*="modal"]').first();
    await expect(modal).toBeVisible({ timeout: 5000 });
  });
});