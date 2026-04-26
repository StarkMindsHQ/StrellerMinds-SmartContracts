#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    UserLayout(Address),
}

#[contract]
pub struct DashboardPreferencesContract;

#[contractimpl]
impl DashboardPreferencesContract {
    /// Saves the user's customized dashboard layout and widget preferences.
    /// The layout is expected to be a serialized string (e.g. JSON)
    /// that the frontend can parse to restore the drag-and-drop state.
    pub fn save_layout(
        env: Env,
        user: Address,
        layout_data: String,
    ) {
        user.require_auth();
        
        env.storage()
            .persistent()
            .set(&DataKey::UserLayout(user), &layout_data);
    }

    /// Retrieves the user's dashboard layout.
    pub fn get_layout(env: Env, user: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::UserLayout(user))
    }

    /// Deletes the user's dashboard layout, reverting to the frontend's default.
    pub fn clear_layout(env: Env, user: Address) {
        user.require_auth();
        
        env.storage()
            .persistent()
            .remove(&DataKey::UserLayout(user));
    }
}

#[cfg(test)]
mod test;