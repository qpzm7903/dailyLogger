use std::str::FromStr;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::memory_storage::DB_CONNECTION;

/// Team visibility level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TeamVisibility {
    Private, // Only members can see
    Public,  // Anyone can see (for future use)
}

impl TeamVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamVisibility::Private => "private",
            TeamVisibility::Public => "public",
        }
    }
}

/// Member role in a team
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Admin,  // Can manage members, delete team
    Member, // Can create/view shared records
    Viewer, // Can only view shared records
}

impl TeamRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamRole::Admin => "admin",
            TeamRole::Member => "member",
            TeamRole::Viewer => "viewer",
        }
    }
}

impl FromStr for TeamRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(TeamRole::Admin),
            "member" => Ok(TeamRole::Member),
            "viewer" => Ok(TeamRole::Viewer),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

/// Team entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub visibility: TeamVisibility,
    pub invite_code: String,
    pub created_at: DateTime<Utc>,
}

/// Team membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub team_id: String,
    pub user_id: String,
    pub username: String,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

/// Team with member info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWithMembers {
    pub team: Team,
    pub members: Vec<TeamMember>,
    pub current_user_role: Option<TeamRole>,
}

/// Create team parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamParams {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<TeamVisibility>,
}

/// Update team parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamParams {
    pub team_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Invite member parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteMemberParams {
    pub team_id: String,
    pub user_id: String,
    pub role: TeamRole,
}

/// Generate unique team ID
fn generate_team_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!(
        "team_{:016x}",
        rng.sample(rand::distributions::Uniform::new(0u64, u64::MAX))
    )
}

/// Generate invite code (8 characters, alphanumeric)
fn generate_invite_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Removed confusing chars: I, O, 0, 1
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.sample(rand::distributions::Uniform::new(0, CHARSET.len()));
            CHARSET[idx] as char
        })
        .collect()
}

/// Create the teams and team_members tables
pub fn create_teams_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            owner_id TEXT NOT NULL REFERENCES users(id),
            visibility TEXT NOT NULL DEFAULT 'private',
            invite_code TEXT UNIQUE NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create teams table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS team_members (
            team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL DEFAULT 'member',
            joined_at TEXT NOT NULL,
            PRIMARY KEY (team_id, user_id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create team_members table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_teams_owner ON teams(owner_id)",
        [],
    )
    .map_err(|e| format!("Failed to create teams owner index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_teams_invite_code ON teams(invite_code)",
        [],
    )
    .map_err(|e| format!("Failed to create invite_code index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_team_members_user ON team_members(user_id)",
        [],
    )
    .map_err(|e| format!("Failed to create team_members user index: {}", e))?;

    Ok(())
}

/// Create a new team (creator becomes admin)
#[tauri::command]
pub fn create_team(params: CreateTeamParams, current_user_id: String) -> Result<Team, String> {
    // Validate name
    if params.name.is_empty() || params.name.len() > 100 {
        return Err("Team name must be 1-100 characters".to_string());
    }

    let team_id = generate_team_id();
    let invite_code = generate_invite_code();
    let created_at = Utc::now().to_rfc3339();
    let visibility = params.visibility.unwrap_or(TeamVisibility::Private);

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Create team
    conn.execute(
        "INSERT INTO teams (id, name, description, owner_id, visibility, invite_code, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            team_id,
            params.name,
            params.description,
            current_user_id,
            visibility.as_str(),
            invite_code,
            created_at
        ],
    )
    .map_err(|e| format!("Failed to create team: {}", e))?;

    // Add creator as admin member
    let joined_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO team_members (team_id, user_id, role, joined_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            team_id,
            current_user_id,
            TeamRole::Admin.as_str(),
            joined_at
        ],
    )
    .map_err(|e| format!("Failed to add creator as member: {}", e))?;

    Ok(Team {
        id: team_id,
        name: params.name,
        description: params.description,
        owner_id: current_user_id,
        visibility,
        invite_code,
        created_at: Utc::now(),
    })
}

/// Get a team by ID
#[tauri::command]
pub fn get_team(team_id: String) -> Result<Option<Team>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let result = conn
        .query_row(
            "SELECT id, name, description, owner_id, visibility, invite_code, created_at
             FROM teams WHERE id = ?1",
            params![team_id],
            |row| {
                let visibility_str: String = row.get(4)?;
                let visibility = match visibility_str.as_str() {
                    "public" => TeamVisibility::Public,
                    _ => TeamVisibility::Private,
                };
                Ok(Team {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    visibility,
                    invite_code: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result)
}

/// Get team by invite code
#[tauri::command]
pub fn get_team_by_invite_code(invite_code: String) -> Result<Option<Team>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let result = conn
        .query_row(
            "SELECT id, name, description, owner_id, visibility, invite_code, created_at
             FROM teams WHERE invite_code = ?1",
            params![invite_code],
            |row| {
                let visibility_str: String = row.get(4)?;
                let visibility = match visibility_str.as_str() {
                    "public" => TeamVisibility::Public,
                    _ => TeamVisibility::Private,
                };
                Ok(Team {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    visibility,
                    invite_code: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result)
}

/// Update team details
#[tauri::command]
pub fn update_team(params: UpdateTeamParams, current_user_id: String) -> Result<Team, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if user is admin
    let role: Option<String> = conn
        .query_row(
            "SELECT role FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![params.team_id, current_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    let role = role.ok_or("You are not a member of this team")?;
    if role != "admin" {
        return Err("Only admins can update team details".to_string());
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    let mut update_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(name) = &params.name {
        if name.is_empty() || name.len() > 100 {
            return Err("Team name must be 1-100 characters".to_string());
        }
        updates.push("name = ?");
        update_params.push(Box::new(name.clone()));
    }

    if let Some(description) = &params.description {
        updates.push("description = ?");
        update_params.push(Box::new(description.clone()));
    }

    if updates.is_empty() {
        return Err("No fields to update".to_string());
    }

    update_params.push(Box::new(params.team_id.clone()));
    let sql = format!("UPDATE teams SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&sql, rusqlite::params_from_iter(update_params.iter()))
        .map_err(|e| format!("Failed to update team: {}", e))?;

    drop(db);
    get_team(params.team_id)?.ok_or("Team not found after update".to_string())
}

/// Delete a team (owner only)
#[tauri::command]
pub fn delete_team(team_id: String, current_user_id: String) -> Result<bool, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if user is owner
    let owner_id: String = conn
        .query_row(
            "SELECT owner_id FROM teams WHERE id = ?1",
            params![team_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Team not found")?;

    if owner_id != current_user_id {
        return Err("Only the team owner can delete the team".to_string());
    }

    let rows_affected = conn
        .execute("DELETE FROM teams WHERE id = ?1", params![team_id])
        .map_err(|e| format!("Failed to delete team: {}", e))?;

    Ok(rows_affected > 0)
}

/// Join a team using invite code
#[tauri::command]
pub fn join_team(invite_code: String, current_user_id: String) -> Result<Team, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Get team by invite code
    let team: Team = conn
        .query_row(
            "SELECT id, name, description, owner_id, visibility, invite_code, created_at
             FROM teams WHERE invite_code = ?1",
            params![invite_code],
            |row| {
                let visibility_str: String = row.get(4)?;
                let visibility = match visibility_str.as_str() {
                    "public" => TeamVisibility::Public,
                    _ => TeamVisibility::Private,
                };
                Ok(Team {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    visibility,
                    invite_code: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Invalid invite code")?;

    // Check if already a member
    let existing: bool = conn
        .query_row(
            "SELECT 1 FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team.id, current_user_id],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?
        .unwrap_or(false);

    if existing {
        return Err("You are already a member of this team".to_string());
    }

    // Add as member
    let joined_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO team_members (team_id, user_id, role, joined_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            team.id,
            current_user_id,
            TeamRole::Member.as_str(),
            joined_at
        ],
    )
    .map_err(|e| format!("Failed to join team: {}", e))?;

    Ok(team)
}

/// Leave a team
#[tauri::command]
pub fn leave_team(team_id: String, current_user_id: String) -> Result<bool, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if user is owner
    let owner_id: Option<String> = conn
        .query_row(
            "SELECT owner_id FROM teams WHERE id = ?1",
            params![team_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some(owner) = owner_id {
        if owner == current_user_id {
            return Err(
                "Team owner cannot leave. Transfer ownership or delete the team instead."
                    .to_string(),
            );
        }
    }

    let rows_affected = conn
        .execute(
            "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, current_user_id],
        )
        .map_err(|e| format!("Failed to leave team: {}", e))?;

    Ok(rows_affected > 0)
}

/// Invite a user to join a team
#[tauri::command]
pub fn invite_member(
    params: InviteMemberParams,
    current_user_id: String,
) -> Result<TeamMember, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if current user is admin
    let role: Option<String> = conn
        .query_row(
            "SELECT role FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![params.team_id, current_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    let role = role.ok_or("You are not a member of this team")?;
    if role != "admin" {
        return Err("Only admins can invite members".to_string());
    }

    // Check if target user exists and get username
    let username: String = conn
        .query_row(
            "SELECT username FROM users WHERE id = ?1",
            params![params.user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("User not found")?;

    // Check if already a member
    let existing: bool = conn
        .query_row(
            "SELECT 1 FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![params.team_id, params.user_id],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?
        .unwrap_or(false);

    if existing {
        return Err("User is already a member of this team".to_string());
    }

    // Add member
    let joined_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO team_members (team_id, user_id, role, joined_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            params.team_id,
            params.user_id,
            params.role.as_str(),
            joined_at
        ],
    )
    .map_err(|e| format!("Failed to add member: {}", e))?;

    Ok(TeamMember {
        team_id: params.team_id,
        user_id: params.user_id,
        username,
        role: params.role,
        joined_at: Utc::now(),
    })
}

/// Update member role
#[tauri::command]
pub fn update_member_role(
    team_id: String,
    user_id: String,
    new_role: TeamRole,
    current_user_id: String,
) -> Result<TeamMember, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if current user is admin
    let role: Option<String> = conn
        .query_row(
            "SELECT role FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, current_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    let role = role.ok_or("You are not a member of this team")?;
    if role != "admin" {
        return Err("Only admins can update member roles".to_string());
    }

    // Cannot change owner's role
    let owner_id: String = conn
        .query_row(
            "SELECT owner_id FROM teams WHERE id = ?1",
            params![team_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    if user_id == owner_id {
        return Err("Cannot change the team owner's role".to_string());
    }

    // Update role
    conn.execute(
        "UPDATE team_members SET role = ?1 WHERE team_id = ?2 AND user_id = ?3",
        params![new_role.as_str(), team_id, user_id],
    )
    .map_err(|e| format!("Failed to update role: {}", e))?;

    // Get username
    let username: String = conn
        .query_row(
            "SELECT username FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(TeamMember {
        team_id,
        user_id,
        username,
        role: new_role,
        joined_at: Utc::now(),
    })
}

/// Remove a member from team
#[tauri::command]
pub fn remove_member(
    team_id: String,
    user_id: String,
    current_user_id: String,
) -> Result<bool, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if current user is admin
    let role: Option<String> = conn
        .query_row(
            "SELECT role FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, current_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    let role = role.ok_or("You are not a member of this team")?;
    if role != "admin" {
        return Err("Only admins can remove members".to_string());
    }

    // Cannot remove owner
    let owner_id: String = conn
        .query_row(
            "SELECT owner_id FROM teams WHERE id = ?1",
            params![team_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    if user_id == owner_id {
        return Err("Cannot remove the team owner".to_string());
    }

    let rows_affected = conn
        .execute(
            "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, user_id],
        )
        .map_err(|e| format!("Failed to remove member: {}", e))?;

    Ok(rows_affected > 0)
}

/// Get all teams the user is a member of
#[tauri::command]
pub fn get_user_teams(user_id: String) -> Result<Vec<TeamWithMembers>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Get all teams where user is a member
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.description, t.owner_id, t.visibility, t.invite_code, t.created_at, tm.role
             FROM teams t
             JOIN team_members tm ON t.id = tm.team_id
             WHERE tm.user_id = ?1
             ORDER BY t.created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let teams_data: Vec<(Team, String)> = stmt
        .query_map(params![user_id], |row| {
            let visibility_str: String = row.get(4)?;
            let visibility = match visibility_str.as_str() {
                "public" => TeamVisibility::Public,
                _ => TeamVisibility::Private,
            };
            let team = Team {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                owner_id: row.get(3)?,
                visibility,
                invite_code: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            };
            let role: String = row.get(7)?;
            Ok((team, role))
        })
        .map_err(|e| format!("Failed to query teams: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect teams: {}", e))?;

    drop(stmt);

    let mut result = Vec::new();
    for (team, user_role) in teams_data {
        // Get all members for this team
        let mut members_stmt = conn
            .prepare(
                "SELECT tm.team_id, tm.user_id, u.username, tm.role, tm.joined_at
                 FROM team_members tm
                 JOIN users u ON tm.user_id = u.id
                 WHERE tm.team_id = ?1
                 ORDER BY
                     CASE tm.role
                         WHEN 'admin' THEN 1
                         WHEN 'member' THEN 2
                         WHEN 'viewer' THEN 3
                     END,
                     tm.joined_at ASC",
            )
            .map_err(|e| format!("Failed to prepare members statement: {}", e))?;

        let members: Vec<TeamMember> = members_stmt
            .query_map(params![team.id], |row| {
                let role_str: String = row.get(3)?;
                let role = TeamRole::from_str(&role_str).unwrap_or(TeamRole::Member);
                Ok(TeamMember {
                    team_id: row.get(0)?,
                    user_id: row.get(1)?,
                    username: row.get(2)?,
                    role,
                    joined_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })
            .map_err(|e| format!("Failed to query members: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect members: {}", e))?;

        let current_user_role = TeamRole::from_str(&user_role).ok();

        result.push(TeamWithMembers {
            team,
            members,
            current_user_role,
        });
    }

    Ok(result)
}

/// Get team members
#[tauri::command]
pub fn get_team_members(team_id: String) -> Result<Vec<TeamMember>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT tm.team_id, tm.user_id, u.username, tm.role, tm.joined_at
             FROM team_members tm
             JOIN users u ON tm.user_id = u.id
             WHERE tm.team_id = ?1
             ORDER BY
                 CASE tm.role
                     WHEN 'admin' THEN 1
                     WHEN 'member' THEN 2
                     WHEN 'viewer' THEN 3
                 END,
                 tm.joined_at ASC",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let members: Vec<TeamMember> = stmt
        .query_map(params![team_id], |row| {
            let role_str: String = row.get(3)?;
            let role = TeamRole::from_str(&role_str).unwrap_or(TeamRole::Member);
            Ok(TeamMember {
                team_id: row.get(0)?,
                user_id: row.get(1)?,
                username: row.get(2)?,
                role,
                joined_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| format!("Failed to query members: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect members: {}", e))?;

    Ok(members)
}

/// Regenerate invite code for a team
#[tauri::command]
pub fn regenerate_invite_code(team_id: String, current_user_id: String) -> Result<String, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if user is admin
    let role: Option<String> = conn
        .query_row(
            "SELECT role FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, current_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    let role = role.ok_or("You are not a member of this team")?;
    if role != "admin" {
        return Err("Only admins can regenerate invite codes".to_string());
    }

    // Generate new code (ensure uniqueness)
    let mut new_code = generate_invite_code();
    let mut attempts = 0;
    while attempts < 10 {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM teams WHERE invite_code = ?1",
                params![new_code],
                |_| Ok(true),
            )
            .optional()
            .map_err(|e| format!("Database error: {}", e))?
            .unwrap_or(false);

        if !exists {
            break;
        }
        new_code = generate_invite_code();
        attempts += 1;
    }

    if attempts >= 10 {
        return Err("Failed to generate unique invite code".to_string());
    }

    conn.execute(
        "UPDATE teams SET invite_code = ?1 WHERE id = ?2",
        params![new_code, team_id],
    )
    .map_err(|e| format!("Failed to update invite code: {}", e))?;

    Ok(new_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{create_users_table, register_user, RegisterParams};
    use rusqlite::Connection;
    use serial_test::serial;

    fn setup_test_db() {
        let conn = Connection::open_in_memory().unwrap();
        create_users_table(&conn).unwrap();
        create_teams_tables(&conn).unwrap();

        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    fn create_test_user(username: &str) -> String {
        let params = RegisterParams {
            username: username.to_string(),
            email: None,
            password: "password123".to_string(),
        };
        register_user(params).unwrap().id
    }

    #[test]
    fn test_generate_team_id_format() {
        let id = generate_team_id();
        assert!(id.starts_with("team_"));
        assert_eq!(id.len(), 21); // "team_" + 16 hex chars
    }

    #[test]
    fn test_generate_invite_code_format() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_team_role_conversion() {
        assert_eq!(TeamRole::Admin.as_str(), "admin");
        assert_eq!(TeamRole::Member.as_str(), "member");
        assert_eq!(TeamRole::Viewer.as_str(), "viewer");

        assert_eq!(TeamRole::from_str("admin").unwrap(), TeamRole::Admin);
        assert_eq!(TeamRole::from_str("member").unwrap(), TeamRole::Member);
        assert_eq!(TeamRole::from_str("viewer").unwrap(), TeamRole::Viewer);
        assert!(TeamRole::from_str("invalid").is_err());
    }

    #[test]
    #[serial]
    fn test_create_team() {
        setup_test_db();
        let user_id = create_test_user("teamowner");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: Some("A test team".to_string()),
            visibility: Some(TeamVisibility::Private),
        };

        let team = create_team(params, user_id.clone()).unwrap();
        assert_eq!(team.name, "Test Team");
        assert_eq!(team.description, Some("A test team".to_string()));
        assert_eq!(team.owner_id, user_id);
        assert!(team.invite_code.len() == 8);

        // Verify creator is admin member
        let members = get_team_members(team.id.clone()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].user_id, user_id);
        assert_eq!(members[0].role, TeamRole::Admin);
    }

    #[test]
    #[serial]
    fn test_create_team_empty_name() {
        setup_test_db();
        let user_id = create_test_user("teamowner2");

        let params = CreateTeamParams {
            name: "".to_string(),
            description: None,
            visibility: None,
        };

        let result = create_team(params, user_id);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_join_team() {
        setup_test_db();
        let owner_id = create_test_user("teamowner3");
        let member_id = create_test_user("teammember");

        let params = CreateTeamParams {
            name: "Joinable Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id).unwrap();

        // Join with invite code
        let joined = join_team(team.invite_code.clone(), member_id.clone()).unwrap();
        assert_eq!(joined.id, team.id);

        // Verify membership
        let members = get_team_members(team.id.clone()).unwrap();
        assert_eq!(members.len(), 2);

        let new_member = members.iter().find(|m| m.user_id == member_id).unwrap();
        assert_eq!(new_member.role, TeamRole::Member);
    }

    #[test]
    #[serial]
    fn test_join_team_already_member() {
        setup_test_db();
        let owner_id = create_test_user("teamowner4");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();

        // Try to join again as owner
        let result = join_team(team.invite_code.clone(), owner_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already a member"));
    }

    #[test]
    #[serial]
    fn test_leave_team() {
        setup_test_db();
        let owner_id = create_test_user("teamowner5");
        let member_id = create_test_user("leavemember");

        let params = CreateTeamParams {
            name: "Leave Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id).unwrap();
        join_team(team.invite_code.clone(), member_id.clone()).unwrap();

        // Leave team
        let result = leave_team(team.id.clone(), member_id.clone()).unwrap();
        assert!(result);

        // Verify not a member
        let members = get_team_members(team.id.clone()).unwrap();
        assert_eq!(members.len(), 1);
    }

    #[test]
    #[serial]
    fn test_owner_cannot_leave() {
        setup_test_db();
        let owner_id = create_test_user("teamowner6");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();

        let result = leave_team(team.id.clone(), owner_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("owner cannot leave"));
    }

    #[test]
    #[serial]
    fn test_invite_member() {
        setup_test_db();
        let owner_id = create_test_user("teamowner7");
        let invitee_id = create_test_user("inviteduser");

        let params = CreateTeamParams {
            name: "Invite Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();

        let invite_params = InviteMemberParams {
            team_id: team.id.clone(),
            user_id: invitee_id.clone(),
            role: TeamRole::Member,
        };

        let member = invite_member(invite_params, owner_id).unwrap();
        assert_eq!(member.user_id, invitee_id);
        assert_eq!(member.role, TeamRole::Member);
    }

    #[test]
    #[serial]
    fn test_invite_member_non_admin() {
        setup_test_db();
        let owner_id = create_test_user("teamowner8");
        let member_id = create_test_user("regularmember");
        let invitee_id = create_test_user("invitee");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id).unwrap();
        join_team(team.invite_code.clone(), member_id.clone()).unwrap();

        let invite_params = InviteMemberParams {
            team_id: team.id.clone(),
            user_id: invitee_id,
            role: TeamRole::Member,
        };

        let result = invite_member(invite_params, member_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only admins"));
    }

    #[test]
    #[serial]
    fn test_update_member_role() {
        setup_test_db();
        let owner_id = create_test_user("teamowner9");
        let member_id = create_test_user("rolechange");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();
        join_team(team.invite_code.clone(), member_id.clone()).unwrap();

        // Promote to admin
        let updated = update_member_role(
            team.id.clone(),
            member_id.clone(),
            TeamRole::Admin,
            owner_id,
        )
        .unwrap();

        assert_eq!(updated.role, TeamRole::Admin);
    }

    #[test]
    #[serial]
    fn test_remove_member() {
        setup_test_db();
        let owner_id = create_test_user("teamowner10");
        let member_id = create_test_user("removedmember");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();
        join_team(team.invite_code.clone(), member_id.clone()).unwrap();

        let result = remove_member(team.id.clone(), member_id.clone(), owner_id).unwrap();
        assert!(result);

        let members = get_team_members(team.id.clone()).unwrap();
        assert_eq!(members.len(), 1);
    }

    #[test]
    #[serial]
    fn test_delete_team() {
        setup_test_db();
        let owner_id = create_test_user("teamowner11");

        let params = CreateTeamParams {
            name: "Delete Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();

        let result = delete_team(team.id.clone(), owner_id).unwrap();
        assert!(result);

        let found = get_team(team.id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    #[serial]
    fn test_delete_team_non_owner() {
        setup_test_db();
        let owner_id = create_test_user("teamowner12");
        let member_id = create_test_user("deleter");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id).unwrap();
        join_team(team.invite_code.clone(), member_id.clone()).unwrap();

        let result = delete_team(team.id.clone(), member_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("owner"));
    }

    #[test]
    #[serial]
    fn test_get_user_teams() {
        setup_test_db();
        let user_id = create_test_user("teamuser");

        let params1 = CreateTeamParams {
            name: "Team 1".to_string(),
            description: None,
            visibility: None,
        };
        let params2 = CreateTeamParams {
            name: "Team 2".to_string(),
            description: None,
            visibility: None,
        };

        create_team(params1, user_id.clone()).unwrap();
        create_team(params2, user_id.clone()).unwrap();

        let teams = get_user_teams(user_id).unwrap();
        assert_eq!(teams.len(), 2);
        assert!(teams
            .iter()
            .all(|t| t.current_user_role == Some(TeamRole::Admin)));
    }

    #[test]
    #[serial]
    fn test_regenerate_invite_code() {
        setup_test_db();
        let owner_id = create_test_user("teamowner13");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();
        let old_code = team.invite_code.clone();

        let new_code = regenerate_invite_code(team.id.clone(), owner_id).unwrap();
        assert_ne!(new_code, old_code);
        assert_eq!(new_code.len(), 8);
    }

    #[test]
    #[serial]
    fn test_update_team() {
        setup_test_db();
        let owner_id = create_test_user("teamowner14");

        let params = CreateTeamParams {
            name: "Original Name".to_string(),
            description: Some("Original desc".to_string()),
            visibility: None,
        };

        let team = create_team(params, owner_id.clone()).unwrap();

        let update_params = UpdateTeamParams {
            team_id: team.id.clone(),
            name: Some("New Name".to_string()),
            description: Some("New description".to_string()),
        };

        let updated = update_team(update_params, owner_id).unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.description, Some("New description".to_string()));
    }

    #[test]
    #[serial]
    fn test_update_team_non_admin() {
        setup_test_db();
        let owner_id = create_test_user("teamowner15");
        let member_id = create_test_user("updater");

        let params = CreateTeamParams {
            name: "Test Team".to_string(),
            description: None,
            visibility: None,
        };

        let team = create_team(params, owner_id).unwrap();
        join_team(team.invite_code.clone(), member_id.clone()).unwrap();

        let update_params = UpdateTeamParams {
            team_id: team.id.clone(),
            name: Some("Hacked Name".to_string()),
            description: None,
        };

        let result = update_team(update_params, member_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only admins"));
    }
}
