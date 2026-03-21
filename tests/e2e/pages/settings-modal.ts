/**
 * 设置弹窗 Page Object Model
 */

import type { Page, Locator } from '@playwright/test';

export class SettingsModal {
  readonly page: Page;

  // 弹窗容器
  readonly modal: Locator;

  // 标签页
  readonly basicTab: Locator;
  readonly aiTab: Locator;
  readonly captureTab: Locator;
  readonly outputTab: Locator;

  // 基础设置输入
  readonly apiUrlInput: Locator;
  readonly apiKeyInput: Locator;
  readonly modelSelect: Locator;

  // 按钮
  readonly saveButton: Locator;
  readonly cancelButton: Locator;
  readonly closeButton: Locator;

  constructor(page: Page) {
    this.page = page;

    // 弹窗
    this.modal = page.locator('[data-testid="settings-modal"], .modal:has-text("设置"), .modal:has-text("Settings")');

    // 标签页
    this.basicTab = this.modal.locator('[data-testid="tab-basic"], button:has-text("Basic"), button:has-text("基础")');
    this.aiTab = this.modal.locator('[data-testid="tab-ai"], button:has-text("AI"), button:has-text("AI")');
    this.captureTab = this.modal.locator('[data-testid="tab-capture"], button:has-text("Capture"), button:has-text("捕获")');
    this.outputTab = this.modal.locator('[data-testid="tab-output"], button:has-text("Output"), button:has-text("输出")');

    // 基础设置输入
    this.apiUrlInput = this.modal.locator('[data-testid="api-url"], input[name="api_base_url"], input[placeholder*="API"]');
    this.apiKeyInput = this.modal.locator('[data-testid="api-key"], input[name="api_key"], input[type="password"]');
    this.modelSelect = this.modal.locator('[data-testid="model-select"], select[name="model_name"], select');

    // 按钮
    this.saveButton = this.modal.locator('[data-testid="save-button"], button:has-text("保存"), button:has-text("Save")');
    this.cancelButton = this.modal.locator('[data-testid="cancel-button"], button:has-text("取消"), button:has-text("Cancel")');
    this.closeButton = this.modal.locator('[data-testid="close-button"], button[aria-label*="close"], button:has([class*="close"])');
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
   * 切换到基础设置标签页
   */
  async goToBasicTab() {
    await this.basicTab.click();
  }

  /**
   * 切换到 AI 设置标签页
   */
  async goToAITab() {
    await this.aiTab.click();
  }

  /**
   * 切换到捕获设置标签页
   */
  async goToCaptureTab() {
    await this.captureTab.click();
  }

  /**
   * 切换到输出设置标签页
   */
  async goToOutputTab() {
    await this.outputTab.click();
  }

  /**
   * 设置 API URL
   */
  async setApiUrl(url: string) {
    await this.apiUrlInput.fill(url);
  }

  /**
   * 设置 API Key
   */
  async setApiKey(key: string) {
    await this.apiKeyInput.fill(key);
  }

  /**
   * 选择模型
   */
  async selectModel(model: string) {
    await this.modelSelect.selectOption({ label: model });
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
   * 点击关闭按钮
   */
  async clickClose() {
    await this.closeButton.first().click();
  }

  /**
   * 获取当前 API URL 值
   */
  async getApiUrl(): Promise<string> {
    return this.apiUrlInput.inputValue();
  }

  /**
   * 完整的设置保存流程
   */
  async saveSettings(settings: { apiUrl?: string; apiKey?: string; model?: string }) {
    await this.waitForVisible();
    if (settings.apiUrl !== undefined) {
      await this.setApiUrl(settings.apiUrl);
    }
    if (settings.apiKey !== undefined) {
      await this.setApiKey(settings.apiKey);
    }
    if (settings.model !== undefined) {
      await this.selectModel(settings.model);
    }
    await this.clickSave();
  }

  /**
   * 检查弹窗是否可见
   */
  async isVisible(): Promise<boolean> {
    return this.modal.isVisible();
  }
}