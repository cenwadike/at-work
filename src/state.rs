use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Profile {
    pub address: String,
    pub profile_type: ProfileType,
    pub name: String,
    pub bio: String,
    pub skills: Vec<String>,
    pub hourly_rate: Uint128,
    pub rating: Option<Rating>,
    pub completed_jobs: u64,
    pub created_at: u64,
    pub social_links: Vec<SocialLink>,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ProfileType {
    Client,
    Freelancer,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rating {
    pub average: f64,
    pub total_reviews: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SocialLink {
    pub platform: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Job {
    pub id: String,
    pub client: Addr,
    pub freelancer: Option<Addr>,
    pub title: String,
    pub description: String,
    pub budget: Uint128,
    pub status: JobStatus,
    pub deadline: u64,
    pub deliverables: Vec<String>,
    pub submission: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum JobStatus {
    Open,
    InProgress,
    UnderReview,
    Completed,
    Disputed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    pub platform_fee: Decimal, // in basis points (1/100 of a percent)
    pub escrow_address: Addr,
    pub token_address: Addr,
}

// Storage items
pub const STATE: Item<State> = Item::new("state");
pub const JOBS: Map<&str, Job> = Map::new("jobs"); // job_id -> Job
pub const PROPOSALS: Map<(&str, &Addr), String> = Map::new("proposals"); // (job_id, freelancer_addr) -> proposal
pub const PROFILES: Map<&str, Profile> = Map::new("profiles");
