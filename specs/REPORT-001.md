# REPORT-001: 周报生成

## 功能需求

生成周工作总结报告，汇总本周所有记录并调用 AI 生成结构化 Markdown 周报。

## 接口定义

### Tauri Commands

#### `generate_weekly_report`

生成周报并返回文件路径。

**参数**: 无

**返回**: `Result<String, String>` - 周报文件路径或错误信息

#### `get_default_weekly_report_prompt`

获取默认周报 Prompt 模板。

**返回**: `String` - 默认 Prompt 模板

### 数据库扩展

Settings 表新增字段：

| 字段 | 类型 | 默认值 | 说明 |
|-----|------|-------|------|
| `weekly_report_prompt` | TEXT | NULL | 自定义周报 Prompt |
| `weekly_report_day` | INTEGER | 0 | 周起始日 (0=周一, 6=周日) |

## 验收条件 (Given/When/Then)

### AC1: 正常生成周报

**Given** 用户有本周的记录
**When** 用户点击"生成周报"
**Then** 系统汇总本周记录并调用 AI 生成结构化 Markdown 周报

### AC2: 周报生成成功

**Given** 周报生成成功
**When** 用户查看
**Then** 显示文件路径并提供打开选项

### AC3: 周报生成失败

**Given** 周报生成失败
**When** 错误发生
**Then** 显示具体错误信息

### AC4: 本周无记录

**Given** 本周无记录
**When** 用户点击生成周报
**Then** 提示"本周无记录"

### AC5: 自定义周报模板

**Given** 用户有自定义周报模板
**When** 生成周报
**Then** 使用自定义模板

### AC6: 打开周报文件

**Given** 周报生成完成
**When** 用户选择打开
**Then** 在默认应用中打开周报文件

## 技术约束

1. 复用 `synthesis/mod.rs` 的日报生成模式
2. 时区处理使用 `and_local_timezone(chrono::Local)`
3. 使用全局 `DB_CONNECTION` Mutex 访问数据库
4. 错误处理使用 `Result<T, String>`

## 周报输出格式

```markdown
# 周报 - YYYY-MM-DD to YYYY-MM-DD

## 本周概览
- 工作天数：X 天
- 记录条数：Y 条
- 截图数量：Z 张

## 每日工作

### 周一 (YYYY-MM-DD)
- ...

### 周二 (YYYY-MM-DD)
- ...

## 本周成果
1. ...
2. ...

## 遇到的问题
- ...

## 下周计划
- ...
```

## 文件命名

周报文件名格式: `周报-{start_date}-to-{end_date}.md`

示例: `周报-2026-03-10-to-2026-03-16.md`