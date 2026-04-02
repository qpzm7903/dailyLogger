/**
 * E2E 测试基础 Fixture
 * 扩展 Playwright test，自动注入 Tauri IPC mock
 */

import { test as base, expect, type Page } from '@playwright/test';
import { getMockInjectionScript, MockOverrides, resetMockState, getMockState, setMockState } from './tauri-mock';
import * as Factory from './test-data';

// ============================================
// 扩展的 Test Fixture
// ============================================

export type TauriTestFixtures = {
  /**
   * Mock 覆盖配置
   * 在测试中覆盖特定命令的返回值
   */
  tauriMock: MockOverrides;

  /**
   * 辅助方法：重置 mock 状态
   */
  resetState: () => void;

  /**
   * 辅助方法：获取当前 mock 状态
   */
  getState: () => ReturnType<typeof getMockState>;

  /**
   * 辅助方法：设置 mock 状态
   */
  setState: (state: Parameters<typeof setMockState>[0]) => void;
};

export const test = base.extend<TauriTestFixtures>({
  // Mock 覆盖配置（测试级 option）
  tauriMock: [{}, { option: true }],

  // 自动注入 mock 脚本
  page: async ({ page, tauriMock }, use) => {
    // 在页面加载前注入 mock 脚本
    await page.addInitScript((mockOverrides) => {
      // 注入到 window 对象，供 getMockInjectionScript 使用
      (window as unknown as Record<string, unknown>).__TAURI_MOCK_OVERRIDES__ = mockOverrides;
    }, tauriMock);

    // 注入 Tauri mock
    await page.addInitScript(getMockInjectionScript(tauriMock));

    await use(page);
  },

  // 重置状态方法
  resetState: async ({}, use) => {
    await use(() => {
      resetMockState();
    });
  },

  // 获取状态方法
  getState: async ({}, use) => {
    await use(() => getMockState());
  },

  // 设置状态方法
  setState: async ({}, use) => {
    await use((state) => setMockState(state));
  },
});

// 重新导出 expect
export { expect };

// 重新导出工厂函数
export { Factory };

// ============================================
// 测试辅助函数
// ============================================

/**
 * 等待应用加载完成
 */
export async function waitForAppReady(page: Page) {
  // 等待主内容区域可见
  await page.waitForSelector('[data-testid="app-container"], main, .app-container', {
    timeout: 10000,
  });
}

/**
 * 打开快速笔记弹窗
 */
export async function openQuickNoteModal(page: Page) {
  // 点击快速笔记按钮
  await page.click('[data-testid="quick-note-button"], button:has-text("快速记录"), button:has-text("Quick")');
  // 等待弹窗出现
  await page.waitForSelector('[data-testid="quick-note-modal"], .modal:has-text("快速")', {
    timeout: 5000,
  });
}

/**
 * 打开设置弹窗
 */
export async function openSettingsModal(page: Page) {
  // 点击设置按钮
  await page.click('[data-testid="settings-button"], button:has([class*="settings"]), button:has-text("设置"), button:has-text("Settings"), button[aria-label*="设置"], button[aria-label*="Settings"]');
  // 等待弹窗出现
  await page.waitForSelector('[data-testid="settings-modal"], .modal:has-text("设置"), .modal:has-text("Settings")', {
    timeout: 5000,
  });
}

/**
 * 关闭弹窗（点击关闭按钮或按 Escape）
 */
export async function closeModal(page: Page) {
  const closeButton = page.locator('[data-testid="close-button"], button:has-text("关闭"), button:has-text("Cancel"), button[aria-label*="close"]');
  const count = await closeButton.count();
  if (count > 0) {
    await closeButton.first().click();
  } else {
    await page.keyboard.press('Escape');
  }
}

/**
 * 检查离线横幅是否显示
 */
export async function isOfflineBannerVisible(page: Page): Promise<boolean> {
  const banner = page.locator('[data-testid="offline-banner"], .offline-banner, [class*="offline"]');
  return banner.isVisible();
}
