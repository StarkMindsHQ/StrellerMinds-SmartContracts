use soroban_sdk::{contracttype, Address, BytesN, String};

/// Social media platform options for sharing.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharePlatform {
    Twitter,
    LinkedIn,
    Facebook,
}

/// Record of a share event for an achievement/credential.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShareRecord {
    /// ID of the certificate/achievement being shared.
    pub certificate_id: BytesN<32>,
    /// Address of the user who shared.
    pub user: Address,
    /// Platform where the share occurred.
    pub platform: SharePlatform,
    /// Custom message included with the share.
    pub custom_message: String,
    /// Generated share URL.
    pub share_url: String,
    /// Unix timestamp of when the share occurred.
    pub timestamp: u64,
    /// Count of engagement (likes, comments, shares).
    pub engagement_count: u32,
    /// Whether the share was verified/confirmed.
    pub verified: bool,
}

/// Analytics data for social sharing.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialSharingAnalytics {
    /// Total number of shares recorded.
    pub total_shares: u32,
    /// Number of shares per platform.
    pub twitter_shares: u32,
    pub linkedin_shares: u32,
    pub facebook_shares: u32,
    /// Total engagement across all platforms.
    pub total_engagement: u32,
    /// Average engagement per share.
    pub average_engagement: u32,
    /// Number of unique users who have shared.
    pub unique_sharers: u32,
    /// Last updated timestamp.
    pub last_updated: u64,
}

impl SocialSharingAnalytics {
    pub fn new() -> Self {
        SocialSharingAnalytics {
            total_shares: 0,
            twitter_shares: 0,
            linkedin_shares: 0,
            facebook_shares: 0,
            total_engagement: 0,
            average_engagement: 0,
            unique_sharers: 0,
            last_updated: 0,
        }
    }
}
