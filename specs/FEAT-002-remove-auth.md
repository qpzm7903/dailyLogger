# FEAT-002: 移除登录/注册功能

## 背景

用户反馈 (#55): 完全不需要登录、注册功能，请去除。DailyLogger 作为个人桌面应用，团队协作功能不是核心需求，移除可以简化代码和维护成本。

## 需求

### 功能需求

1. 移除所有登录/注册相关的前端组件和 UI 入口
2. 移除所有认证相关的后端代码
3. 移除团队协作功能（依赖认证模块）
4. 清理相关的数据库表结构定义
5. 清理相关的国际化文本

### 非功能需求

- 保持应用其他功能正常运作
- 确保移除后无编译警告/错误
- 更新相关测试用例

## 技术方案

### 后端移除清单

**删除文件:**
| 文件 | 行数 | 说明 |
|------|------|------|
| `src-tauri/src/auth/mod.rs` | ~750 | 认证模块 |
| `src-tauri/src/team/mod.rs` | ~1900 | 团队协作模块 |

**修改文件:**
| 文件 | 修改内容 |
|------|----------|
| `src-tauri/src/lib.rs` | 移除 `pub mod auth;` 和 `pub mod team;` |
| `src-tauri/src/main.rs` | 移除 generate_handler![] 中所有 auth/team 命令 |
| `src-tauri/src/memory_storage/schema.rs` | 移除 users, sessions, teams, team_members, shared_records 表 |

**移除的 Tauri 命令 (23个):**
```
register_user, login_user, get_user_by_id, get_all_users,
delete_user, has_any_user, get_current_session, logout,
create_team, get_team, get_team_by_invite_code, update_team,
delete_team, join_team, leave_team, invite_member,
update_member_role, remove_member, get_user_teams,
get_team_members, regenerate_invite_code, share_record_to_team,
unshare_record_from_team, get_team_shared_records, get_user_shared_records
```

### 前端移除清单

**删除文件:**
| 文件 | 行数 | 说明 |
|------|------|------|
| `src/components/LoginModal.vue` | ~200 | 登录/注册模态框 |
| `src/components/TeamPanel.vue` | ~510 | 团队管理面板 |

**修改文件:**
| 文件 | 修改内容 |
|------|----------|
| `src/App.vue` | 移除 auth 相关导入、currentUser、登录按钮、TeamPanel |
| `src/composables/useModal.ts` | 移除 'loginModal' 和 'teamPanel' 类型 |
| `src/types/tauri.ts` | 移除 Team, TeamMember, User, LoginArgs, RegisterArgs, AuthResult 类型 |
| `src/components/HistoryViewer.vue` | 移除 currentUser prop |
| `src/locales/en.json` | 移除 auth.* 和 team.* 键 |
| `src/locales/zh-CN.json` | 移除 auth.* 和 team.* 键 |

### 数据库表移除

**不再创建的表:**
```sql
users           -- 用户表
sessions        -- 会话表
teams           -- 团队表
team_members    -- 团队成员表
shared_records  -- 共享记录表
```

**保留的表:**
```sql
records         -- 核心记录表（不受影响）
settings        -- 设置表（不受影响）
```

## 验收标准

### Given/When/Then

**场景 1: 应用启动无认证**
- Given: 用户启动应用
- When: 应用加载完成
- Then: 不显示登录按钮，直接进入主界面

**场景 2: 构建无警告**
- Given: 开发者运行 cargo build
- When: 编译完成
- Then: 无 auth/team 相关的未使用警告

**场景 3: 测试通过**
- Given: 运行所有测试
- When: 测试完成
- Then: 所有测试通过，无 auth/team 相关测试失败

## 影响范围

### 移除的功能
- 用户注册/登录
- 用户会话管理
- 团队创建/管理
- 团队邀请/加入
- 记录共享

### 不受影响的功能
- 自动截图捕获
- 手动记录
- AI 日报/周报/月报生成
- Obsidian/Logseq/Notion 导出
- GitHub 工时统计
- Slack 通知
- 全文搜索
- 数据备份/恢复

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 用户数据丢失 | 低 | auth/team 数据独立，不影响 records |
| 构建失败 | 中 | 分步移除，每步验证编译 |
| 遗留代码 | 低 | 全局搜索 auth/team 引用 |

## 测试用例

### 后端测试

```rust
// 确保无 auth/team 模块引用
#[test]
fn test_no_auth_module() {
    // 验证 auth 模块不存在
}

#[test]
fn test_no_team_module() {
    // 验证 team 模块不存在
}
```

### 前端测试

```typescript
describe('Auth removal', () => {
  it('should not show login button', () => {})
  it('should not have loginModal in useModal', () => {})
  it('should not have teamPanel in useModal', () => {})
})
```

## 估时

- 后端移除: 1pt
- 前端移除: 1pt
- 测试修复: 0.5pt
- 验证 CI: 0.5pt
- **总计: 3pts**