/**
 * 快速笔记弹窗 Page Object Model
 */

import type { Page, Locator } from '@playwright/test';

export class QuickNoteModal {
  readonly page: Page;

  // 弹窗容器
  readonly modal: Locator;

  // 输入元素
  readonly contentInput: Locator;
  readonly tagInput: Locator;

  // 按钮
  readonly saveButton: Locator;
  readonly cancelButton: Locator;

  constructor(page: Page) {
    this.page = page;

    // 弹窗
    this.modal = page.locator('[data-testid="quick-note-modal"], .modal:has-text("快速"), .modal:has-text("Quick")');

    // 输入
    this.contentInput = this.modal.locator('textarea, [data-testid="note-content"], [contenteditable="true"]');
    this.tagInput = this.modal.locator('[data-testid="tag-input"], input[placeholder*="标签"], input[placeholder*="tag"]');

    // 按钮
    this.saveButton = this.modal.locator('[data-testid="save-button"], button:has-text("保存"), button:has-text("Save")');
    this.cancelButton = this.modal.locator('[data-testid="cancel-button"], button:has-text("取消"), button:has-text("Cancel")');
  }

  /**
   * 等待弹窗出现
   */
  async waitForVisible() {
    await this.modal.waitFor({ state: 'visible', timeout: 5000 });
  }

  /**
   * 等待弹窗消失
   */
  async waitForHidden() {
    await this.modal.waitFor({ state: 'hidden', timeout: 5000 });
  }

  /**
   * 输入笔记内容
   */
  async enterContent(content: string) {
    await this.contentInput.fill(content);
  }

  /**
   * 添加标签
   */
  async addTag(tag: string) {
    await this.tagInput.fill(tag);
    await this.tagInput.press('Enter');
  }

  /**
   * 点击保存按钮
   */
  async clickSave() {
    await this.saveButton.click();
  }

  /**
   * 点击取消按钮
   */
  async clickCancel() {
    await this.cancelButton.click();
  }

  /**
   * 完整的创建笔记流程
   */
  async createNote(content: string, tags: string[] = []) {
    await this.waitForVisible();
    await this.enterContent(content);
    for (const tag of tags) {
      await this.addTag(tag);
    }
    await this.clickSave();
    await this.waitForHidden();
  }

  /**
   * 检查弹窗是否可见
   */
  async isVisible(): Promise<boolean> {
    return this.modal.isVisible();
  }
}