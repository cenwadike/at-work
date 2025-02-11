use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::state::{Job, JobStatus, Profile, ProfileType, SocialLink};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub platform_fee: u64,
    pub escrow_address: Addr,
    pub token_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateProfile {
        profile_type: ProfileType,
        name: String,
        bio: String,
        skills: Vec<String>,
        hourly_rate: Uint128,
        social_links: Vec<SocialLink>,
    },
    UpdateProfile {
        name: Option<String>,
        bio: Option<String>,
        skills: Option<Vec<String>>,
        hourly_rate: Option<Uint128>,
        social_links: Option<Vec<SocialLink>>,
    },
    PostJob {
        title: String,
        description: String,
        budget: Uint128,
        deadline: u64,
        deliverables: Vec<String>,
    },
    ApplyForJob {
        job_id: String,
        proposal: String,
    },
    AcceptProposal {
        job_id: String,
        freelancer: Addr,
    },
    SubmitWork {
        job_id: String,
        submission: String,
    },
    ApproveWork {
        job_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetProfileResponse)]
    GetProfile { address: String },
    #[returns(GetListProfilesResponse)]
    ListProfiles {
        profile_type: Option<ProfileType>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(GetSearchFreelancersResponse)]
    SearchFreelancers {
        skills: Vec<String>,
        min_rating: Option<f64>,
        max_hourly_rate: Option<Uint128>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(GetJobResponse)]
    GetJob { job_id: String },
    #[returns(GetListJobsResponse)]
    ListJobs {
        status: Option<JobStatus>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(GetFreelancerJobsResponse)]
    GetFreelancerJobs {
        address: Addr,
        status: Option<JobStatus>,
    },
    #[returns(GetClientJobsResponse)]
    GetClientJobs {
        address: Addr,
        status: Option<JobStatus>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetProfileResponse {
    pub profile: Profile,
}

#[cw_serde]
pub struct GetListProfilesResponse {
    pub profiles: Vec<Profile>,
}

#[cw_serde]
pub struct GetSearchFreelancersResponse {
    pub profiles: Vec<Profile>,
}

#[cw_serde]
pub struct GetJobResponse {
    pub job: Job,
}

#[cw_serde]
pub struct GetListJobsResponse {
    pub jobs: Vec<Job>,
}

#[cw_serde]
pub struct GetFreelancerJobsResponse {
    pub jobs: Vec<Job>,
}

#[cw_serde]
pub struct GetClientJobsResponse {
    pub jobs: Vec<Job>,
}
