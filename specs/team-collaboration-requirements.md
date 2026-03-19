# Team Collaboration Mode - Requirements Definition

> Status: Draft
> Created: 2026-03-19
> Target Version: v2.0.0+

## Overview

Enable multiple users to collaborate on work logs and reports, sharing records within teams while maintaining individual privacy controls.

## Current Architecture Constraints

- Single-user desktop app with local SQLite storage
- No authentication system
- No cloud storage or sync mechanism
- Records are private by design

## MVP Feature Scope

### 1. User Authentication

**Requirement**: Support user identity for team membership.

**Options**:
- A) Local user accounts with username/password
- B) OAuth integration (Google, GitHub, Microsoft)
- C) Self-hosted auth server (Keycloak, Authentik)

**Recommendation**: Start with Option A (local accounts) for simplicity, add OAuth later.

### 2. Team Management

**Requirement**: Allow users to create teams and invite members.

**Features**:
- Create team with name and description
- Generate invite link or code
- Invite via email (optional)
- Member roles: Admin, Member, Viewer
- Leave / remove from team

### 3. Data Sync Architecture

**Requirement**: Sync records across team members.

**Options**:
- A) Cloud storage (AWS S3, Cloudflare R2)
- B) Self-hosted sync server
- C) P2P sync (libp2p, without central server)
- D) Database-as-a-service (Supabase, Firebase)

**Recommendation**: Option D (Supabase) for rapid development with built-in auth and real-time sync.

### 4. Permission Model

**Requirement**: Control who can see which records.

**Record visibility levels**:
- **Private**: Only the author can see
- **Team**: All team members can see
- **Public**: Anyone with the link can see (optional)

**Default**: Records are private unless explicitly shared.

### 5. Conflict Resolution

**Requirement**: Handle concurrent edits.

**Strategy**:
- Last-write-wins for simple fields
- Merge strategy for content fields
- Version history for recovery

## Non-Goals for MVP

- Real-time collaborative editing
- End-to-end encryption
- Mobile team features
- Enterprise SSO

## Technical Considerations

### Backend Changes

1. **Authentication Module** (`src-tauri/src/auth/`)
   - User registration / login
   - Session management
   - Token refresh

2. **Sync Module** (`src-tauri/src/sync/`)
   - Record sync queue
   - Conflict detection
   - Offline queue

3. **Team Module** (`src-tauri/src/team/`)
   - Team CRUD
   - Membership management
   - Role-based access control

### Database Schema Changes

```sql
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Teams table
CREATE TABLE teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    owner_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL
);

-- Team memberships
CREATE TABLE team_members (
    team_id TEXT REFERENCES teams(id),
    user_id TEXT REFERENCES users(id),
    role TEXT NOT NULL DEFAULT 'member', -- 'admin', 'member', 'viewer'
    joined_at TEXT NOT NULL,
    PRIMARY KEY (team_id, user_id)
);

-- Records visibility
ALTER TABLE records ADD COLUMN visibility TEXT DEFAULT 'private';
ALTER TABLE records ADD COLUMN author_id TEXT REFERENCES users(id);
ALTER TABLE records ADD COLUMN team_id TEXT REFERENCES teams(id);
```

### Frontend Changes

1. **Login/Register Page** - New authentication flow
2. **Team Management UI** - Settings modal extension
3. **Record Sharing UI** - Visibility toggle on records
4. **Team Dashboard** - View team activity

## Implementation Phases

### Phase 1: Authentication (2-3 story points)
- User registration / login
- Session persistence
- Logout

### Phase 2: Team CRUD (3 story points)
- Create / join / leave teams
- Invite members
- Role management

### Phase 3: Record Sharing (3 story points)
- Visibility controls
- Team record creation
- Permission checks

### Phase 4: Sync (5 story points)
- Cloud storage integration
- Conflict resolution
- Offline support

## Open Questions

1. **Cloud Provider**: Which sync backend to use? (Supabase recommended)
2. **Pricing Model**: Free tier limits? Self-hosted option?
3. **Data Migration**: How to migrate existing single-user data to team mode?
4. **Multi-tenancy**: Support multiple teams per user?

## Next Steps

1. Evaluate Supabase vs self-hosted options
2. Create auth module prototype
3. Design API contracts for team operations
4. Update plan.md with implementation timeline