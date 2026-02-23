use crate::types::*;
use soroban_sdk::{Env, String, Vec};

pub struct ApiDocManager;

impl ApiDocManager {
    pub fn create_endpoint(
        env: &Env,
        endpoint_id: String,
        name: String,
        description: String,
        method: String,
        path: String,
        parameters: Vec<ApiParameter>,
        response_schema: String,
        version: String,
    ) -> Result<ApiEndpoint, Error> {
        if env.storage().persistent().has(&DataKey::ApiEndpoint(endpoint_id.clone())) {
            return Err(Error::AlreadyExists);
        }

        let endpoint = ApiEndpoint {
            endpoint_id: endpoint_id.clone(),
            name,
            description,
            method,
            path,
            parameters,
            response_schema,
            code_examples: Vec::new(env),
            version,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ApiEndpoint(endpoint_id), &endpoint);

        Ok(endpoint)
    }

    pub fn add_code_example(
        env: &Env,
        endpoint_id: String,
        example: CodeExample,
    ) -> Result<(), Error> {
        let mut endpoint: ApiEndpoint = env
            .storage()
            .persistent()
            .get(&DataKey::ApiEndpoint(endpoint_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        endpoint.code_examples.push_back(example);
        env.storage()
            .persistent()
            .set(&DataKey::ApiEndpoint(endpoint_id), &endpoint);

        Ok(())
    }

    pub fn get_endpoint(env: &Env, endpoint_id: &String) -> Option<ApiEndpoint> {
        env.storage()
            .persistent()
            .get(&DataKey::ApiEndpoint(endpoint_id.clone()))
    }
}
