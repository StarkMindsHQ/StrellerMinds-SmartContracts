#![no_std]
extern crate alloc;

use soroban_sdk::{Env, String, Vec};

/// CORS configuration for external academic verification services
#[derive(Clone, Debug, PartialEq)]
pub struct CorsConfig {
    /// Allowed origins for cross-origin requests
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Exposed headers
    pub exposed_headers: Vec<String>,
    /// Maximum age for preflight requests (in seconds)
    pub max_age: u64,
    /// Whether credentials are allowed
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self::permissive()
    }
}

impl CorsConfig {
    /// Creates a permissive CORS configuration for development
    pub fn permissive() -> Self {
        Self {
            allowed_origins: Vec::from_array(&Env::current(), [
                String::from_str(&Env::current(), "*"),
            ]),
            allowed_methods: Vec::from_array(&Env::current(), [
                String::from_str(&Env::current(), "GET"),
                String::from_str(&Env::current(), "POST"),
                String::from_str(&Env::current(), "PUT"),
                String::from_str(&Env::current(), "DELETE"),
                String::from_str(&Env::current(), "OPTIONS"),
            ]),
            allowed_headers: Vec::from_array(&Env::current(), [
                String::from_str(&Env::current(), "Content-Type"),
                String::from_str(&Env::current(), "Authorization"),
                String::from_str(&Env::current(), "X-Requested-With"),
                String::from_str(&Env::current(), "Accept"),
                String::from_str(&Env::current(), "Origin"),
            ]),
            exposed_headers: Vec::from_array(&Env::current(), [
                String::from_str(&Env::current(), "X-Total-Count"),
                String::from_str(&Env::current(), "X-Page-Count"),
            ]),
            max_age: 86400, // 24 hours
            allow_credentials: false,
        }
    }

    /// Creates a restrictive CORS configuration for production
    pub fn restrictive(env: &Env, allowed_domains: &[String]) -> Self {
        let mut origins = Vec::new(env);
        for domain in allowed_domains {
            origins.push_back(domain.clone());
        }

        Self {
            allowed_origins: origins,
            allowed_methods: Vec::from_array(env, [
                String::from_str(env, "GET"),
                String::from_str(env, "POST"),
                String::from_str(env, "OPTIONS"),
            ]),
            allowed_headers: Vec::from_array(env, [
                String::from_str(env, "Content-Type"),
                String::from_str(env, "Authorization"),
            ]),
            exposed_headers: Vec::new(env),
            max_age: 3600, // 1 hour
            allow_credentials: true,
        }
    }

    /// Creates a CORS configuration specifically for academic verification services
    pub fn academic_verification(env: &Env) -> Self {
        Self {
            allowed_origins: Vec::from_array(env, [
                String::from_str(env, "https://*.edu"),
                String::from_str(env, "https://*.ac.*"),
                String::from_str(env, "https://starkminds.io"),
                String::from_str(env, "https://localhost:*"),
                String::from_str(env, "http://localhost:*"),
            ]),
            allowed_methods: Vec::from_array(env, [
                String::from_str(env, "GET"),
                String::from_str(env, "POST"),
                String::from_str(env, "OPTIONS"),
            ]),
            allowed_headers: Vec::from_array(env, [
                String::from_str(env, "Content-Type"),
                String::from_str(env, "Authorization"),
                String::from_str(env, "X-API-Key"),
                String::from_str(env, "X-Verification-Token"),
                String::from_str(env, "Accept"),
            ]),
            exposed_headers: Vec::from_array(env, [
                String::from_str(env, "X-Verification-Status"),
                String::from_str(env, "X-Retry-After"),
                String::from_str(env, "X-Rate-Limit-Remaining"),
            ]),
            max_age: 7200, // 2 hours
            allow_credentials: true,
        }
    }

    /// Validates if an origin is allowed
    pub fn is_origin_allowed(&self, env: &Env, origin: &String) -> bool {
        for allowed_origin in self.allowed_origins.iter() {
            if allowed_origin == String::from_str(env, "*") {
                return true;
            }
            if self.matches_pattern(env, allowed_origin, origin) {
                return true;
            }
        }
        false
    }

    /// Validates if a method is allowed
    pub fn is_method_allowed(&self, env: &Env, method: &String) -> bool {
        for allowed_method in self.allowed_methods.iter() {
            if allowed_method == method {
                return true;
            }
        }
        false
    }

    /// Validates if a header is allowed
    pub fn is_header_allowed(&self, env: &Env, header: &String) -> bool {
        for allowed_header in self.allowed_headers.iter() {
            if allowed_header == header {
                return true;
            }
        }
        false
    }

    /// Pattern matching for wildcard origins
    fn matches_pattern(&self, env: &Env, pattern: &String, origin: &String) -> bool {
        let pattern_bytes = pattern.to_bytes();
        let origin_bytes = origin.to_bytes();
        
        // Simple wildcard matching for now
        // In a real implementation, you'd want more sophisticated pattern matching
        if pattern_bytes.len() > origin_bytes.len() {
            return false;
        }
        
        // Check if pattern ends with wildcard
        if pattern_bytes.len() > 2 {
            let last_two = pattern_bytes.slice(pattern_bytes.len() - 2..);
            if last_two == soroban_sdk::Bytes::from_slice(env, b"*") {
                let prefix = pattern_bytes.slice(0..pattern_bytes.len() - 2);
                let origin_prefix = origin_bytes.slice(0..prefix.len());
                return prefix == origin_prefix;
            }
        }
        
        pattern == origin
    }
}

/// CORS headers for HTTP responses
#[derive(Clone, Debug)]
pub struct CorsHeaders {
    pub access_control_allow_origin: String,
    pub access_control_allow_methods: String,
    pub access_control_allow_headers: String,
    pub access_control_expose_headers: String,
    pub access_control_max_age: String,
    pub access_control_allow_credentials: String,
}

impl CorsHeaders {
    /// Generates CORS headers based on configuration and request
    pub fn generate(env: &Env, config: &CorsConfig, origin: &Option<String>, method: &Option<String>) -> Self {
        let allow_origin = if let Some(req_origin) = origin {
            if config.is_origin_allowed(env, req_origin) {
                req_origin.clone()
            } else {
                String::from_str(env, "null")
            }
        } else {
            String::from_str(env, "*")
        };

        let allow_methods = self::join_strings(env, &config.allowed_methods, ", ");
        let allow_headers = self::join_strings(env, &config.allowed_headers, ", ");
        let expose_headers = self::join_strings(env, &config.exposed_headers, ", ");
        let max_age = String::from_slice(env, &config.max_age.to_string().as_bytes());
        let allow_credentials = String::from_str(env, if config.allow_credentials { "true" } else { "false" });

        Self {
            access_control_allow_origin: allow_origin,
            access_control_allow_methods: allow_methods,
            access_control_allow_headers: allow_headers,
            access_control_expose_headers: expose_headers,
            access_control_max_age: max_age,
            access_control_allow_credentials: allow_credentials,
        }
    }
}

/// Helper function to join strings with a separator
fn join_strings(env: &Env, strings: &Vec<String>, separator: &str) -> String {
    let mut result = String::new(env);
    let sep = String::from_str(env, separator);
    
    let mut first = true;
    for s in strings.iter() {
        if !first {
            result = result.concat(&sep);
        }
        result = result.concat(s);
        first = false;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cors_config() {
        let env = Env::default();
        let config = CorsConfig::default();
        
        assert_eq!(config.allowed_origins.len(), 1);
        assert_eq!(config.allowed_methods.len(), 5);
        assert_eq!(config.allowed_headers.len(), 5);
        assert_eq!(config.max_age, 86400);
        assert!(!config.allow_credentials);
    }

    #[test]
    fn test_academic_verification_config() {
        let env = Env::default();
        let config = CorsConfig::academic_verification(&env);
        
        assert_eq!(config.allowed_origins.len(), 5);
        assert_eq!(config.allowed_methods.len(), 3);
        assert_eq!(config.allowed_headers.len(), 5);
        assert_eq!(config.max_age, 7200);
        assert!(config.allow_credentials);
    }

    #[test]
    fn test_origin_validation() {
        let env = Env::default();
        let config = CorsConfig::academic_verification(&env);
        
        let edu_origin = String::from_str(&env, "https://university.edu");
        assert!(config.is_origin_allowed(&env, &edu_origin));
        
        let invalid_origin = String::from_str(&env, "https://malicious.com");
        assert!(!config.is_origin_allowed(&env, &invalid_origin));
    }
}
