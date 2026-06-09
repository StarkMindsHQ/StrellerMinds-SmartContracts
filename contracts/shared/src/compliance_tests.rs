#[cfg(test)]
mod tests {
    use crate::roles::{RoleLevel, Permission};
    use crate::permissions::RolePermissions;
    use soroban_sdk::{Env, Vec};

    #[test]
    fn test_compliance_admin_role_level() {
        let level = RoleLevel::ComplianceAdmin;
        assert_eq!(level.to_u32(), 6);
        assert_eq!(RoleLevel::from_u32(6), Some(RoleLevel::ComplianceAdmin));
    }

    #[test]
    fn test_compliance_admin_permissions() {
        let env = Env::default();
        let permissions = RolePermissions::compliance_admin_permissions(&env);
        
        assert!(permissions.contains(&Permission::GenerateComplianceReport));
        assert!(permissions.contains(&Permission::ViewAudit));
        assert!(permissions.contains(&Permission::ViewSystemStats));
    }

    #[test]
    fn test_admin_can_grant_compliance_admin() {
        let admin = RoleLevel::Admin;
        let compliance_admin = RoleLevel::ComplianceAdmin;
        
        // Admin (4) can grant ComplianceAdmin (6) due to explicit logic in can_grant
        assert!(admin.can_grant(&compliance_admin));
    }

    #[test]
    fn test_super_admin_can_grant_compliance_admin() {
        let super_admin = RoleLevel::SuperAdmin;
        let compliance_admin = RoleLevel::ComplianceAdmin;
        
        assert!(super_admin.can_grant(&compliance_admin));
    }

    #[test]
    fn test_moderator_cannot_grant_compliance_admin() {
        let moderator = RoleLevel::Moderator;
        let compliance_admin = RoleLevel::ComplianceAdmin;
        
        assert!(!moderator.can_grant(&compliance_admin));
    }
}
