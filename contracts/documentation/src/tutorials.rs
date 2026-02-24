use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

pub struct TutorialManager;

impl TutorialManager {
    pub fn create_tutorial(
        env: &Env,
        tutorial_id: String,
        title: String,
        description: String,
        difficulty: DifficultyLevel,
        estimated_time: u32,
        author: &Address,
        steps: Vec<TutorialStep>,
        prerequisites: Vec<String>,
    ) -> Result<Tutorial, Error> {
        if env
            .storage()
            .persistent()
            .has(&DataKey::Tutorial(tutorial_id.clone()))
        {
            return Err(Error::AlreadyExists);
        }

        let tutorial = Tutorial {
            tutorial_id: tutorial_id.clone(),
            title,
            description,
            difficulty,
            estimated_time,
            author: author.clone(),
            steps,
            prerequisites,
            created_at: env.ledger().timestamp(),
            completion_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Tutorial(tutorial_id), &tutorial);

        Ok(tutorial)
    }

    pub fn complete_tutorial(env: &Env, tutorial_id: String) -> Result<(), Error> {
        let mut tutorial: Tutorial = env
            .storage()
            .persistent()
            .get(&DataKey::Tutorial(tutorial_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        tutorial.completion_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Tutorial(tutorial_id), &tutorial);

        Ok(())
    }

    pub fn get_tutorial(env: &Env, tutorial_id: &String) -> Option<Tutorial> {
        env.storage()
            .persistent()
            .get(&DataKey::Tutorial(tutorial_id.clone()))
    }
}
