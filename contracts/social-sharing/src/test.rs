#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Address, Env, BytesN, String};
    use crate::types::SharePlatform;

    #[test]
    fn test_init_contract() {
        let env = Env::default();
        let admin = Address::random(&env);
        
        let result = SocialSharingContract::init_contract(env.clone(), admin.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn test_share_achievement() {
        let env = Env::default();
        let admin = Address::random(&env);
        let user = Address::random(&env);
        
        // Initialize contract
        SocialSharingContract::init_contract(env.clone(), admin.clone()).unwrap();
        
        // Create a share
        let cert_id = BytesN::from_array(&env, &[1u8; 32]);
        let message = String::from_slice(&env, "Check out my achievement!");
        
        let result = SocialSharingContract::share_achievement(
            env.clone(),
            user.clone(),
            cert_id.clone(),
            SharePlatform::Twitter,
            message,
        );
        
        assert!(result.is_ok());
        let share_record = result.unwrap();
        assert_eq!(share_record.certificate_id, cert_id);
        assert_eq!(share_record.user, user);
    }

    #[test]
    fn test_get_analytics() {
        let env = Env::default();
        let admin = Address::random(&env);
        
        SocialSharingContract::init_contract(env.clone(), admin).unwrap();
        
        let result = SocialSharingContract::get_analytics(env);
        assert!(result.is_ok());
        
        let analytics = result.unwrap();
        assert_eq!(analytics.total_shares, 0);
    }
}
