# DailyLogger E2E Tests

AI-driven end-to-end testing framework for DailyLogger Tauri application.

## 概述

这个测试框架使用 Playwright 连接到 Tauri 应用的 Chrome DevTools Protocol (CDP) 端口，然后通过 AI Agent 自主执行测试任务。

**核心流程：**
```
Perceive (感知) → Think (思考) → Act (行动) → Verify (验证)
      ↑                                              │
      └──────────────── 循环 ─────────────────────────┘
```

## 环境要求

- Python 3.10+
- Node.js 18+
- Rust (用于编译 Tauri)
- OpenAI API Key (或兼容的 API)

## 快速开始

### 1. 安装依赖

```bash
cd e2e-tests

# 使用 uv 创建虚拟环境并安装依赖
uv venv
uv sync

# 安装 Playwright 浏览器
.venv/Scripts/python.exe -m playwright install chromium
```

### 2. 配置环境变量

```bash
# 复制示例配置
cp .env.example .env

# 编辑 .env 文件，填入你的 API Key
# OPENAI_API_KEY=sk-your-key-here
```

### 3. 启动 Tauri 应用（开启 CDP 端口）

**Windows PowerShell:**
```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
npm run tauri dev
```

**注意：** 应用必须在测试运行前启动。

### 4. 运行测试

```bash
# 激活虚拟环境
.venv\Scripts\activate

# 运行交互式测试
python run_e2e_test.py
```

## 项目结构

```
e2e-tests/
├── pyproject.toml        # 项目配置 (uv)
├── .env.example          # 环境变量模板
├── run_e2e_test.py       # 主测试入口
├── tauri_connection.py   # Tauri CDP 连接模块
├── ui_state_extractor.py # UI 状态提取模块
├── agent_loop.py         # AI Agent 核心循环
├── test_cases.py         # 测试用例定义
├── tests/                # 框架单元测试
│   └── test_framework.py
└── screenshots/          # 测试截图输出
```

## 模块说明

### tauri_connection.py

负责通过 CDP 连接到 Tauri 应用：

```python
from tauri_connection import connect_to_tauri

page, browser = await connect_to_tauri(cdp_port=9222)
```

### ui_state_extractor.py

从页面提取简化的 UI 状态，供 LLM 分析：

```python
from ui_state_extractor import extract_ui_state

state = await extract_ui_state(page)
print(state.to_description())  # LLM 可读的 UI 描述
```

### agent_loop.py

实现 AI Agent 的"感知-思考-行动"循环：

```python
from agent_loop import run_test

result = await run_test(
    task="Test the quick note feature",
    cdp_port=9222
)
```

## 编写自定义测试

### 方式一：交互式输入

运行 `python run_e2e_test.py`，选择 [5] Custom Task，输入测试任务。

### 方式二：在 test_cases.py 添加测试用例

```python
TestCase(
    id="TC007",
    name="My Custom Test",
    description="Description of the test",
    task="""Detailed steps:
    1. Click button X
    2. Enter text Y
    3. Verify result Z
    """,
    expected_outcome="Something happens"
)
```

### 方式三：直接调用 API

```python
import asyncio
from agent_loop import AITestAgent
from tauri_connection import connect_to_tauri

async def my_test():
    page, browser = await connect_to_tauri()
    agent = AITestAgent(model="gpt-4o")
    
    result = await agent.run(
        page,
        task="Your test task description"
    )
    
    print(f"Success: {result.success}")
    print(f"Steps: {result.total_steps}")

asyncio.run(my_test())
```

## 可用的 Agent 动作

| 动作 | 参数 | 说明 |
|------|------|------|
| `click` | target | 点击元素 |
| `type` | target, value | 在输入框输入文本 |
| `clear` | target | 清空输入框 |
| `scroll` | value (up/down) | 滚动页面 |
| `wait` | value (秒数或选择器) | 等待 |
| `verify` | description | 验证条件 |
| `done` | description | 测试完成 |
| `fail` | description | 测试失败 |

## 故障排查

### 连接失败

```
❌ Tauri app not running or CDP port not enabled
```

确保：
1. Tauri 应用正在运行
2. 环境变量 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 已设置
3. 端口 9222 未被其他程序占用

### API 错误

```
LLM error: ...
```

检查：
1. `.env` 中的 `OPENAI_API_KEY` 是否正确
2. 如果使用自定义 API，检查 `OPENAI_API_BASE` 配置

### 元素未找到

Agent 报告找不到元素时：
1. 检查 `screenshots/` 目录中的截图
2. 确认应用界面是否有变化
3. 尝试用更具体的元素描述

## 调试技巧

### 查看截图

每个步骤都会在 `screensshots/` 目录保存截图：

```
screenshots/
├── step-1.png
├── step-2.png
└── ...
```

### 打印 UI 状态

```python
state = await extract_ui_state(page)
print(state.to_description())
```

### 检查 Accessibility Tree

```python
tree = await get_accessibility_tree(page)
print(tree)
```

## 运行框架测试

```bash
# 使用 pytest 测试框架本身
.venv\Scripts\python.exe -m pytest tests/
```