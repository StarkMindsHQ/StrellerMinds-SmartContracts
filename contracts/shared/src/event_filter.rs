use crate::event_schema::StandardEvent;
use soroban_sdk::{Address, Env, Symbol, Vec};

/// Event filter criteria
#[derive(Clone, Debug)]
pub struct EventFilter {
    /// Filter by event categories
    pub categories: Option<Vec<Symbol>>,
    /// Filter by event types
    pub event_types: Option<Vec<Symbol>>,
    /// Filter by contract addresses
    pub contracts: Option<Vec<Symbol>>,
    /// Filter by actor addresses
    pub actors: Option<Vec<Address>>,
    /// Filter by timestamp range (start, end)
    pub timestamp_range: Option<(u64, u64)>,
    /// Filter by sequence range (start, end)
    pub sequence_range: Option<(u32, u32)>,
}

/// Event router for directing events to different handlers
pub struct EventRouter;

impl EventRouter {
    /// Route an event based on filter criteria
    pub fn route(env: &Env, event: &StandardEvent, filter: &EventFilter) -> bool {
        Self::matches_filter(env, event, filter)
    }

    /// Check if an event matches the filter criteria
    pub fn matches_filter(_env: &Env, event: &StandardEvent, filter: &EventFilter) -> bool {
        // Check category filter
        if let Some(ref categories) = filter.categories {
            let event_category = event.get_category();
            let mut matches = false;
            for cat in categories.iter() {
                if cat.to_string() == event_category {
                    matches = true;
                    break;
                }
            }
            if !matches {
                return false;
            }
        }

        // Check event type filter
        if let Some(ref event_types) = filter.event_types {
            let event_type = event.get_event_type();
            let mut matches = false;
            for et in event_types.iter() {
                if et.to_string() == event_type {
                    matches = true;
                    break;
                }
            }
            if !matches {
                return false;
            }
        }

        // Check contract filter
        if let Some(ref contracts) = filter.contracts {
            let mut matches = false;
            for contract in contracts.iter() {
                if contract == event.contract {
                    matches = true;
                    break;
                }
            }
            if !matches {
                return false;
            }
        }

        // Check actor filter
        if let Some(ref actors) = filter.actors {
            let mut matches = false;
            for actor in actors.iter() {
                if actor == event.actor {
                    matches = true;
                    break;
                }
            }
            if !matches {
                return false;
            }
        }

        // Check timestamp range
        if let Some((start, end)) = filter.timestamp_range {
            if event.timestamp < start || event.timestamp > end {
                return false;
            }
        }

        true
    }

    /// Create a filter for a specific category
    pub fn category_filter(env: &Env, category: Symbol) -> EventFilter {
        let mut categories = Vec::new(env);
        categories.push_back(category);
        EventFilter {
            categories: Some(categories),
            event_types: None,
            contracts: None,
            actors: None,
            timestamp_range: None,
            sequence_range: None,
        }
    }

    /// Create a filter for a specific contract
    pub fn contract_filter(env: &Env, contract: Symbol) -> EventFilter {
        let mut contracts = Vec::new(env);
        contracts.push_back(contract);
        EventFilter {
            categories: None,
            event_types: None,
            contracts: Some(contracts),
            actors: None,
            timestamp_range: None,
            sequence_range: None,
        }
    }

    /// Create a filter for a specific actor
    pub fn actor_filter(env: &Env, actor: Address) -> EventFilter {
        let mut actors = Vec::new(env);
        actors.push_back(actor);
        EventFilter {
            categories: None,
            event_types: None,
            contracts: None,
            actors: Some(actors),
            timestamp_range: None,
            sequence_range: None,
        }
    }

    /// Create a filter for a time range
    pub fn time_range_filter(start: u64, end: u64) -> EventFilter {
        EventFilter {
            categories: None,
            event_types: None,
            contracts: None,
            actors: None,
            timestamp_range: Some((start, end)),
            sequence_range: None,
        }
    }

    /// Create a combined filter
    pub fn combined_filter(
        _env: &Env,
        categories: Option<Vec<Symbol>>,
        contracts: Option<Vec<Symbol>>,
        actors: Option<Vec<Address>>,
        timestamp_range: Option<(u64, u64)>,
    ) -> EventFilter {
        EventFilter {
            categories,
            event_types: None,
            contracts,
            actors,
            timestamp_range,
            sequence_range: None,
        }
    }
}

/// Event filter builder for fluent API
pub struct EventFilterBuilder {
    filter: EventFilter,
}

impl EventFilterBuilder {
    pub fn new() -> Self {
        EventFilterBuilder {
            filter: EventFilter {
                categories: None,
                event_types: None,
                contracts: None,
                actors: None,
                timestamp_range: None,
                sequence_range: None,
            },
        }
    }

    pub fn with_categories(mut self, _env: &Env, categories: Vec<Symbol>) -> Self {
        self.filter.categories = Some(categories);
        self
    }

    pub fn with_event_types(mut self, _env: &Env, event_types: Vec<Symbol>) -> Self {
        self.filter.event_types = Some(event_types);
        self
    }

    pub fn with_contracts(mut self, _env: &Env, contracts: Vec<Symbol>) -> Self {
        self.filter.contracts = Some(contracts);
        self
    }

    pub fn with_actors(mut self, _env: &Env, actors: Vec<Address>) -> Self {
        self.filter.actors = Some(actors);
        self
    }

    pub fn with_timestamp_range(mut self, start: u64, end: u64) -> Self {
        self.filter.timestamp_range = Some((start, end));
        self
    }

    pub fn with_sequence_range(mut self, start: u32, end: u32) -> Self {
        self.filter.sequence_range = Some((start, end));
        self
    }

    pub fn build(self) -> EventFilter {
        self.filter
    }
}
