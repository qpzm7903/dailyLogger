/**
 * 主页面 Page Object Model
 */

import type { Page, Locator } from '@playwright/test';

export class MainPage {
  readonly page: Page;

  // Header 元素
  readonly header: Locator;
  readonly quickNoteButton: Locator;
  readonly settingsButton: Locator;
  readonly reportDropdown: Locator;

  // 主内容区域
  readonly recordList: Locator;
  readonly tagFilter: Locator;
  readonly screenshotGallery: Locator;

  // 状态指示器
  readonly offlineBanner: Locator;
  readonly captureStatus: Locator;

  constructor(page: Page) {
    this.page = page;

    // Header - 使用实际的元素选择器
    this.header = page.locator('header');
    this.quickNoteButton = page.locator('button:has-text("记录")').first();
    this.settingsButton = page
      .locator('[data-testid="settings-button"]')
      .or(page.locator('aside button').filter({ hasText: /设置|Settings/ }))
      .or(page.locator('aside button[title="设置"], aside button[title="Settings"]'));
    this.reportDropdown = page.locator('.report-dropdown, [class*="report"]').first();

    // 内容区域 - 今日工作流区域
    this.recordList = page.locator('h2:has-text("今日工作流")').locator('..');
    this.tagFilter = page.locator('[class*="tag"], .tag-filter').first();
    this.screenshotGallery = page.locator('[class*="screenshot"]').first();

    // 状态
    this.offlineBanner = page.locator('.offline-banner, [class*="offline"]').first();
    this.captureStatus = page.locator('[class*="capture"]').first();
  }

  /**
   * 导航到主页面
   */
  async goto() {
    await this.page.goto('/');
  }

  /**
   * 等待页面加载完成
   */
  async waitForReady() {
    // 等待 header 可见
    await this.header.waitFor({ state: 'visible', timeout: 15000 });
    // 等待主要内容区域可见
    await this.page.locator('text=今日工作流').waitFor({ state: 'visible', timeout: 15000 });
  }

  /**
   * 点击快速记录按钮
   */
  async clickQuickNote() {
    // 点击带有 "记录" 文本的按钮
    await this.page.locator('button:has-text("记录")').first().click();
  }

  /**
   * 点击设置按钮
   */
  async clickSettings() {
    // 点击设置按钮 (settings button is in sidebar)
    await this.settingsButton.click();
  }

  /**
   * 获取记录数量
   */
  async getRecordCount(): Promise<number> {
    const records = this.page.locator('[class*="record-item"], [class*="workflow-item"]');
    return records.count();
  }

  /**
   * 检查离线横幅是否可见
   */
  async isOffline(): Promise<boolean> {
    try {
      return await this.offlineBanner.isVisible();
    } catch {
      return false;
    }
  }

  /**
   * 点击报告下拉菜单
   */
  async openReportDropdown() {
    await this.reportDropdown.click();
  }

  /**
   * 选择报告类型
   */
  async selectReportType(type: 'daily' | 'weekly' | 'monthly') {
    await this.openReportDropdown();
    const text = type === 'daily' ? '日报' : type === 'weekly' ? '周报' : '月报';
    const option = this.page.locator(`button:has-text("${text}")`);
    await option.click();
  }
}
