//! Governance, RBAC, and audit logging

use crate::Result;
use std::collections::HashMap;

/// Role-based access control role
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Role {
    Admin,
    Manager,
    Analyst,
    Viewer,
    Custom(String),
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::Manager => "manager",
            Role::Analyst => "analyst",
            Role::Viewer => "viewer",
            Role::Custom(name) => name,
        }
    }
}

/// Action that can be performed
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Action {
    View,
    Create,
    Edit,
    Delete,
    Export,
    Activate,
    Configure,
    Admin,
}

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::View => "view",
            Action::Create => "create",
            Action::Edit => "edit",
            Action::Delete => "delete",
            Action::Export => "export",
            Action::Activate => "activate",
            Action::Configure => "configure",
            Action::Admin => "admin",
        }
    }
}

/// Resource type
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Resource {
    Segment,
    Audience,
    Dashboard,
    Settings,
    AuditLog,
    Integration,
    Custom(String),
}

impl Resource {
    pub fn as_str(&self) -> &str {
        match self {
            Resource::Segment => "segment",
            Resource::Audience => "audience",
            Resource::Dashboard => "dashboard",
            Resource::Settings => "settings",
            Resource::AuditLog => "audit_log",
            Resource::Integration => "integration",
            Resource::Custom(name) => name,
        }
    }
}

/// Permission definition
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Permission {
    pub role: Role,
    pub action: Action,
    pub resource: Resource,
    pub allowed: bool,
}

impl Permission {
    pub fn new(role: Role, action: Action, resource: Resource, allowed: bool) -> Self {
        Self {
            role,
            action,
            resource,
            allowed,
        }
    }
}

/// User identity
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub role: Role,
    pub active: bool,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

impl User {
    pub fn new(user_id: String, email: String, role: Role) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            user_id,
            email,
            role,
            active: true,
            created_at: now,
            last_login: None,
        }
    }
}

/// Audit event
#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub resource_id: String,
    pub status: String, // "success", "failure", "denied"
    pub details: HashMap<String, String>,
}

impl AuditEvent {
    pub fn new(user_id: String, action: String, resource: String, resource_id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let event_id = format!("evt_{}_{}", now, user_id);

        Self {
            event_id,
            timestamp: now,
            user_id,
            action,
            resource,
            resource_id,
            status: "pending".to_string(),
            details: HashMap::new(),
        }
    }

    pub fn success(mut self) -> Self {
        self.status = "success".to_string();
        self
    }

    pub fn denied(mut self) -> Self {
        self.status = "denied".to_string();
        self
    }

    pub fn failure(mut self, reason: String) -> Self {
        self.status = "failure".to_string();
        self.details.insert("reason".to_string(), reason);
        self
    }

    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// Audit log
#[derive(Clone, Debug)]
pub struct AuditLog {
    events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn record(&mut self, event: AuditEvent) -> Result<()> {
        self.events.push(event);
        Ok(())
    }

    pub fn get_events(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn get_user_events(&self, user_id: &str) -> Vec<AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn get_resource_events(&self, resource: &str, resource_id: &str) -> Vec<AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.resource == resource && e.resource_id == resource_id)
            .cloned()
            .collect()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn get_denied_actions(&self) -> Vec<AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.status == "denied")
            .cloned()
            .collect()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// RBAC Policy manager
pub struct RBACManager {
    permissions: HashMap<(Role, Action, Resource), bool>,
    users: HashMap<String, User>,
}

impl RBACManager {
    pub fn new() -> Self {
        let mut manager = Self {
            permissions: HashMap::new(),
            users: HashMap::new(),
        };

        // Set default permissions
        manager.set_default_permissions();
        manager
    }

    fn set_default_permissions(&mut self) {
        // Admin has all permissions
        for action in &[
            Action::View,
            Action::Create,
            Action::Edit,
            Action::Delete,
            Action::Export,
            Action::Activate,
            Action::Configure,
            Action::Admin,
        ] {
            for resource in &[
                Resource::Segment,
                Resource::Audience,
                Resource::Dashboard,
                Resource::Settings,
                Resource::AuditLog,
                Resource::Integration,
            ] {
                self.permissions
                    .insert((Role::Admin, action.clone(), resource.clone()), true);
            }
        }

        // Manager permissions
        let manager_permissions = vec![
            (Action::View, Resource::Segment),
            (Action::Create, Resource::Segment),
            (Action::Edit, Resource::Segment),
            (Action::Export, Resource::Audience),
            (Action::Activate, Resource::Audience),
            (Action::View, Resource::Dashboard),
            (Action::View, Resource::AuditLog),
        ];

        for (action, resource) in manager_permissions {
            self.permissions
                .insert((Role::Manager, action, resource), true);
        }

        // Analyst permissions
        let analyst_permissions = vec![
            (Action::View, Resource::Segment),
            (Action::View, Resource::Dashboard),
            (Action::Export, Resource::Audience),
            (Action::Create, Resource::Audience),
        ];

        for (action, resource) in analyst_permissions {
            self.permissions
                .insert((Role::Analyst, action, resource), true);
        }

        // Viewer permissions (read-only)
        let viewer_permissions = vec![
            (Action::View, Resource::Segment),
            (Action::View, Resource::Dashboard),
            (Action::View, Resource::AuditLog),
        ];

        for (action, resource) in viewer_permissions {
            self.permissions
                .insert((Role::Viewer, action, resource), true);
        }
    }

    /// Register a user
    pub fn register_user(&mut self, user: User) -> Result<()> {
        self.users.insert(user.user_id.clone(), user);
        Ok(())
    }

    /// Get a user
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).cloned()
    }

    /// Check permission
    pub fn check_permission(&self, role: &Role, action: &Action, resource: &Resource) -> bool {
        self.permissions
            .get(&(role.clone(), action.clone(), resource.clone()))
            .copied()
            .unwrap_or(false)
    }

    /// Check if user can perform action
    pub fn can_user_perform(&self, user_id: &str, action: &Action, resource: &Resource) -> bool {
        match self.get_user(user_id) {
            Some(user) if user.active => self.check_permission(&user.role, action, resource),
            _ => false,
        }
    }

    /// Grant permission to role
    pub fn grant_permission(&mut self, role: Role, action: Action, resource: Resource) {
        self.permissions.insert((role, action, resource), true);
    }

    /// Revoke permission from role
    pub fn revoke_permission(&mut self, role: Role, action: Action, resource: Resource) {
        self.permissions.insert((role, action, resource), false);
    }

    /// List users
    pub fn list_users(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }

    /// Deactivate user
    pub fn deactivate_user(&mut self, user_id: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            user.active = false;
        }
        Ok(())
    }
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new(Role::Admin, Action::View, Resource::Segment, true);

        assert_eq!(perm.role, Role::Admin);
        assert!(perm.allowed);
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "user_1".to_string(),
            "test@example.com".to_string(),
            Role::Analyst,
        );

        assert_eq!(user.user_id, "user_1");
        assert!(user.active);
        assert_eq!(user.role, Role::Analyst);
    }

    #[test]
    fn test_audit_event() {
        let event = AuditEvent::new(
            "user_1".to_string(),
            "view".to_string(),
            "segment".to_string(),
            "seg_1".to_string(),
        )
        .success();

        assert_eq!(event.user_id, "user_1");
        assert_eq!(event.status, "success");
    }

    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new();
        let event = AuditEvent::new(
            "user_1".to_string(),
            "view".to_string(),
            "segment".to_string(),
            "seg_1".to_string(),
        );

        log.record(event).unwrap();
        assert_eq!(log.event_count(), 1);
    }

    #[test]
    fn test_rbac_manager_admin() {
        let mut manager = RBACManager::new();
        let admin = User::new(
            "admin_1".to_string(),
            "admin@example.com".to_string(),
            Role::Admin,
        );

        manager.register_user(admin).unwrap();

        assert!(manager.can_user_perform("admin_1", &Action::Delete, &Resource::Segment));
    }

    #[test]
    fn test_rbac_manager_viewer() {
        let mut manager = RBACManager::new();
        let viewer = User::new(
            "viewer_1".to_string(),
            "viewer@example.com".to_string(),
            Role::Viewer,
        );

        manager.register_user(viewer).unwrap();

        assert!(manager.can_user_perform("viewer_1", &Action::View, &Resource::Segment));
        assert!(!manager.can_user_perform("viewer_1", &Action::Delete, &Resource::Segment));
    }

    #[test]
    fn test_grant_revoke_permission() {
        let mut manager = RBACManager::new();

        manager.grant_permission(Role::Viewer, Action::Create, Resource::Audience);
        assert!(manager.check_permission(&Role::Viewer, &Action::Create, &Resource::Audience));

        manager.revoke_permission(Role::Viewer, Action::Create, Resource::Audience);
        assert!(!manager.check_permission(&Role::Viewer, &Action::Create, &Resource::Audience));
    }

    #[test]
    fn test_user_deactivation() {
        let mut manager = RBACManager::new();
        let user = User::new(
            "user_1".to_string(),
            "test@example.com".to_string(),
            Role::Analyst,
        );

        manager.register_user(user).unwrap();
        manager.deactivate_user("user_1").unwrap();

        assert!(!manager.can_user_perform("user_1", &Action::View, &Resource::Segment));
    }

    #[test]
    fn test_audit_log_filtering() {
        let mut log = AuditLog::new();

        let event1 = AuditEvent::new(
            "user_1".to_string(),
            "view".to_string(),
            "segment".to_string(),
            "seg_1".to_string(),
        );
        let event2 = AuditEvent::new(
            "user_2".to_string(),
            "edit".to_string(),
            "segment".to_string(),
            "seg_1".to_string(),
        );

        log.record(event1).unwrap();
        log.record(event2).unwrap();

        assert_eq!(log.get_user_events("user_1").len(), 1);
        assert_eq!(log.get_resource_events("segment", "seg_1").len(), 2);
    }

    #[test]
    fn test_audit_event_denied() {
        let event = AuditEvent::new(
            "user_1".to_string(),
            "delete".to_string(),
            "segment".to_string(),
            "seg_1".to_string(),
        )
        .denied();

        assert_eq!(event.status, "denied");
    }

    #[test]
    fn test_role_string() {
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::Viewer.as_str(), "viewer");
    }

    #[test]
    fn test_list_users() {
        let mut manager = RBACManager::new();
        let user1 = User::new(
            "user_1".to_string(),
            "test1@example.com".to_string(),
            Role::Admin,
        );
        let user2 = User::new(
            "user_2".to_string(),
            "test2@example.com".to_string(),
            Role::Viewer,
        );

        manager.register_user(user1).unwrap();
        manager.register_user(user2).unwrap();

        assert_eq!(manager.list_users().len(), 2);
    }
}
