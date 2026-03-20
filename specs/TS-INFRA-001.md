# v1.30.0 TypeScript Infrastructure Spec

## Overview

为前端引入 TypeScript 基础设施，为后续全量代码迁移做准备。本版本不涉及代码迁移，仅建立基础设施。

## Goals

1. 安装 TypeScript 及相关依赖
2. 创建 TypeScript 配置文件
3. 更新构建和测试工具配置
4. 在 CI 中添加类型检查步骤
5. 验证现有代码在新配置下仍能正常运行

## Requirements

### TS-001: Install TypeScript Dependencies

**Priority**: HIGH
**Story Points**: 1pt

Install the following dev dependencies:
- `typescript` - TypeScript compiler
- `vue-tsc` - Vue TypeScript type checker
- `@vue/tsconfig` - Vue TypeScript config presets
- `@types/node` - Node.js type definitions

**Verification**:
- Given: package.json has no TypeScript dependencies
- When: npm install completes
- Then: package.json devDependencies includes all required packages

---

### TS-002: Create tsconfig.json

**Priority**: HIGH
**Story Points**: 1pt

Create `tsconfig.json` with the following configuration:
- Enable `strict` mode
- Set `allowJs: true` to support gradual migration
- Configure path aliases matching vite.config.js
- Set appropriate `target` and `moduleResolution`

**Configuration**:
```json
{
  "extends": "@vue/tsconfig/tsconfig.dom.json",
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "allowJs": true,
    "checkJs": false,
    "noEmit": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    },
    "types": ["vitest/globals", "node"]
  },
  "include": ["src/**/*.ts", "src/**/*.js", "src/**/*.vue"],
  "exclude": ["node_modules"]
}
```

**Verification**:
- Given: No tsconfig.json exists
- When: tsconfig.json is created with the above config
- Then: Running `npx tsc --noEmit` does not error on config

---

### TS-003: Update vite.config.js for TypeScript Support

**Priority**: HIGH
**Story Points**: 1pt

Update `vite.config.js` to:
- Support TypeScript path aliases
- Add type checking script integration (optional)

**Changes**:
- The current config already supports `.ts` files via Vite's built-in TypeScript support
- Ensure path aliases in vite.config.js match tsconfig.json
- No major changes needed, just verify compatibility

**Verification**:
- Given: vite.config.js exists
- When: TypeScript files are processed
- Then: Vite correctly resolves path aliases

---

### TS-004: Update Vitest Configuration for TypeScript

**Priority**: MEDIUM
**Story Points**: 1pt

Update vitest configuration in `vite.config.js`:
- Test files already include `.ts` pattern in include
- Verify setupFiles path resolution works with TypeScript

**Current config already supports**:
```javascript
include: ['src/**/*.{spec,test}.{js,ts}', 'src/__tests__/**/*.{js,ts}']
```

**Verification**:
- Given: Vitest config supports .ts files
- When: New .ts test files are created
- Then: Vitest runs them correctly

---

### TS-005: Add Type Check Step to CI Workflow

**Priority**: HIGH
**Story Points**: 1pt

Update `.github/workflows/test.yml` to add a TypeScript type check job:

```yaml
typecheck:
  name: TypeScript Type Check
  runs-on: ubuntu-latest

  steps:
    - uses: actions/checkout@v6

    - name: Setup Node.js
      uses: actions/setup-node@v6
      with:
        node-version: '20'
        cache: 'npm'

    - name: Install dependencies
      run: npm ci

    - name: Type check
      run: npx vue-tsc --noEmit
```

**Verification**:
- Given: CI workflow exists
- When: A PR is created or code is pushed
- Then: Type check step runs and passes

---

### TS-006: Verify All Tests Pass

**Priority**: HIGH
**Story Points**: 1pt

After all changes, verify:
- All existing frontend tests pass (531 tests)
- All existing Rust tests pass (480 tests)
- Build succeeds
- Dev server starts correctly

**Verification**:
- Given: All TS infrastructure is in place
- When: Running `npm run test` and `cargo test`
- Then: All tests pass

---

## Implementation Order

1. TS-001: Install dependencies
2. TS-002: Create tsconfig.json
3. TS-003: Verify vite.config.js compatibility
4. TS-004: Verify vitest config compatibility
5. TS-006: Run all tests locally to verify
6. TS-005: Update CI workflow
7. Commit and push, verify CI passes

## Acceptance Criteria

- [ ] TypeScript and related packages are installed
- [ ] tsconfig.json exists with strict mode enabled
- [ ] `npx vue-tsc --noEmit` runs without errors
- [ ] CI workflow includes type check step
- [ ] All existing tests pass
- [ ] No code migration required in this version

## Out of Scope

- Migrating any .js files to .ts
- Adding `lang="ts"` to Vue components
- Creating type definitions for Tauri commands