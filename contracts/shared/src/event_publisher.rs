use soroban_sdk::{Address, BytesN, Env, Symbol, String, Vec, Map};
use crate::event_schema::{StandardEvent, EventData, EventCategory};

/// Event subscription information
#[derive(Clone, Debug)]
pub struct Subscription {
    /// Subscriber contract address
    pub subscriber: Address,
    /// Event categories to subscribe to
    pub categories: Vec<Symbol>,
    /// Event types to filter (empty = all types)
    pub event_types: Vec<Symbol>,
    /// Contract sources to filter (empty = all contracts)
    pub contracts: Vec<Symbol>,
    /// Active status
    pub active: bool,
    /// Subscription creation timestamp
    pub created_at: u64,
}

/// Event publisher/subscriber manager
pub struct EventPublisher;

impl EventPublisher {
    /// Storage keys for subscriptions
    const SUBSCRIPTION_KEY: &'static str = "sub_";
    const SUBSCRIBER_LIST_KEY: &'static str = "subscribers";
    const EVENT_SEQUENCE_KEY: &'static str = "event_seq";
    const MAX_SUBSCRIPTIONS: u32 = 100;

    /// Subscribe to events with filtering criteria
    pub fn subscribe(
        env: &Env,
        subscriber: &Address,
        categories: Vec<Symbol>,
        event_types: Vec<Symbol>,
        contracts: Vec<Symbol>,
    ) -> Result<u32, String> {
        // Validate subscription limits
        let subscriber_count = Self::get_subscriber_count(env);
        if subscriber_count >= Self::MAX_SUBSCRIPTIONS {
            return Err(String::from_str(env, "Max subscriptions reached"));
        }

        // Validate categories
        for cat in categories.iter() {
            if !Self::is_valid_category(env, cat) {
                return Err(String::from_str(env, "Invalid category"));
            }
        }

        // Create subscription
        let subscription = Subscription {
            subscriber: subscriber.clone(),
            categories: categories.clone(),
            event_types: event_types.clone(),
            contracts: contracts.clone(),
            active: true,
            created_at: env.ledger().timestamp(),
        };

        // Store subscription
        let sub_id = Self::get_next_subscription_id(env);
        let key = Self::get_subscription_key(env, sub_id);
        env.storage().persistent().set(&key, &subscription);

        // Add to subscriber list
        Self::add_subscriber(env, subscriber, sub_id);

        Ok(sub_id)
    }

    /// Unsubscribe from events
    pub fn unsubscribe(env: &Env, subscriber: &Address, subscription_id: u32) -> Result<(), String> {
        let key = Self::get_subscription_key(env, subscription_id);
        let mut subscription: Subscription = env.storage()
            .persistent()
            .get(&key)
            .ok_or_else(|| String::from_str(env, "Subscription not found"))?;

        // Verify ownership
        if subscription.subscriber != *subscriber {
            return Err(String::from_str(env, "Not authorized"));
        }

        // Deactivate subscription
        subscription.active = false;
        env.storage().persistent().set(&key, &subscription);

        // Remove from subscriber list
        Self::remove_subscriber(env, subscriber, subscription_id);

        Ok(())
    }

    /// Publish an event and notify all matching subscribers
    pub fn publish(env: &Env, mut event: StandardEvent) -> Result<u32, String> {
        // Get next event sequence for ordering
        let sequence = Self::get_next_event_sequence(env);
        
        // Set sequence on event for ordering guarantees
        event.sequence = Some(sequence);

        // Validate event before publishing
        crate::event_utils::EventUtils::validate_event(env, &event)?;

        // Emit the event to Soroban's event system
        event.emit(env);

        // Notify subscribers (in a real implementation, this would call subscriber contracts)
        // For gas optimization, we only store subscription info and let subscribers query
        Self::store_event_reference(env, sequence, &event);

        Ok(sequence)
    }

    /// Get subscriptions for a subscriber
    pub fn get_subscriptions(env: &Env, subscriber: &Address) -> Vec<u32> {
        let key = Symbol::new(env, &format!("{}subs_{}", Self::SUBSCRIPTION_KEY, subscriber.to_string()));
        env.storage()
            .persistent()
            .get::<_, Vec<u32>>(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Check if an event matches a subscription
    pub fn matches_subscription(
        env: &Env,
        subscription: &Subscription,
        event: &StandardEvent,
    ) -> bool {
        if !subscription.active {
            return false;
        }

        // Check category match
        let event_category = event.get_category();
        let mut category_match = subscription.categories.is_empty();
        for cat in subscription.categories.iter() {
            if cat.to_string() == event_category {
                category_match = true;
                break;
            }
        }
        if !category_match {
            return false;
        }

        // Check event type match
        if !subscription.event_types.is_empty() {
            let event_type = event.get_event_type();
            let mut type_match = false;
            for et in subscription.event_types.iter() {
                if et.to_string() == event_type {
                    type_match = true;
                    break;
                }
            }
            if !type_match {
                return false;
            }
        }

        // Check contract match
        if !subscription.contracts.is_empty() {
            let mut contract_match = false;
            for contract in subscription.contracts.iter() {
                if contract == &event.contract {
                    contract_match = true;
                    break;
                }
            }
            if !contract_match {
                return false;
            }
        }

        true
    }

    // Private helper functions

    fn get_subscription_key(env: &Env, sub_id: u32) -> Symbol {
        Symbol::new(env, &format!("{}sub_{}", Self::SUBSCRIPTION_KEY, sub_id))
    }

    fn get_next_subscription_id(env: &Env) -> u32 {
        let key = Symbol::new(env, "next_sub_id");
        let current: u32 = env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&key, &next);
        next
    }

    fn get_subscriber_count(env: &Env) -> u32 {
        let key = Symbol::new(env, Self::SUBSCRIBER_LIST_KEY);
        env.storage()
            .persistent()
            .get::<_, u32>(&key)
            .unwrap_or(0)
    }

    fn add_subscriber(env: &Env, subscriber: &Address, sub_id: u32) {
        // Add to subscriber's subscription list
        let key = Symbol::new(env, &format!("{}subs_{}", Self::SUBSCRIPTION_KEY, subscriber.to_string()));
        let mut subs = Self::get_subscriptions(env, subscriber);
        subs.push_back(sub_id);
        env.storage().persistent().set(&key, &subs);

        // Update total subscriber count
        let count_key = Symbol::new(env, Self::SUBSCRIBER_LIST_KEY);
        let count = Self::get_subscriber_count(env) + 1;
        env.storage().persistent().set(&count_key, &count);
    }

    fn remove_subscriber(env: &Env, subscriber: &Address, sub_id: u32) {
        let key = Symbol::new(env, &format!("{}subs_{}", Self::SUBSCRIPTION_KEY, subscriber.to_string()));
        let mut subs = Self::get_subscriptions(env, subscriber);
        let mut new_subs = Vec::new(env);
        for s in subs.iter() {
            if s != sub_id {
                new_subs.push_back(s);
            }
        }
        env.storage().persistent().set(&key, &new_subs);
    }

    fn get_next_event_sequence(env: &Env) -> u32 {
        let key = Symbol::new(env, Self::EVENT_SEQUENCE_KEY);
        let current: u32 = env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&key, &next);
        next
    }

    fn store_event_reference(env: &Env, sequence: u32, event: &StandardEvent) {
        // Store minimal event reference for replay (just sequence and timestamp)
        let key = Symbol::new(env, &format!("evt_seq_{}", sequence));
        let reference = (sequence, event.timestamp, event.contract.clone());
        env.storage().temporary().set(&key, &reference);
    }

    fn is_valid_category(env: &Env, category: &Symbol) -> bool {
        let cat_str = category.to_string();
        matches!(
            cat_str.as_str(),
            "access_control" | "certificate" | "analytics" | "token" | "progress" | "system" | "error"
        )
    }
}
