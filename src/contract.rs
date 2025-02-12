#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use execute::{
    execute_accept_proposal, execute_apply_for_job, execute_approve_work, execute_create_profile,
    execute_post_job, execute_submit_work, execute_update_profile,
};
use query::{
    query_client_jobs, query_freelancer_jobs, query_job, query_list_jobs, query_list_profiles,
    query_profile, query_search_freelancers,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:upwork";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        admin: msg.admin.clone(),
        platform_fee: Decimal::percent(msg.platform_fee.clone()),
        escrow_address: msg.escrow_address.clone(),
        token_address: msg.token_address.clone(),
    };

    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", msg.admin.to_string())
        .add_attribute("platform_fee", msg.platform_fee.to_string())
        .add_attribute("escrow_address", msg.escrow_address.to_string())
        .add_attribute("token_address", msg.token_address.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateProfile {
            profile_type,
            name,
            bio,
            skills,
            hourly_rate,
            social_links,
        } => Ok(execute_create_profile(
            deps,
            env,
            info,
            profile_type,
            name,
            bio,
            skills,
            hourly_rate,
            social_links,
        )?),
        ExecuteMsg::UpdateProfile {
            name,
            bio,
            skills,
            hourly_rate,
            social_links,
        } => Ok(execute_update_profile(
            deps,
            env,
            info,
            name,
            bio,
            skills,
            hourly_rate,
            social_links,
        )?),
        ExecuteMsg::PostJob {
            title,
            description,
            budget,
            deadline,
            deliverables,
        } => Ok(execute_post_job(
            deps,
            env,
            info,
            title,
            description,
            budget,
            deadline,
            deliverables,
        )?),
        ExecuteMsg::ApplyForJob { job_id, proposal } => {
            Ok(execute_apply_for_job(deps, env, info, job_id, proposal)?)
        }
        ExecuteMsg::AcceptProposal { job_id, freelancer } => Ok(execute_accept_proposal(
            deps, env, info, job_id, freelancer,
        )?),
        ExecuteMsg::SubmitWork { job_id, submission } => {
            Ok(execute_submit_work(deps, env, info, job_id, submission)?)
        }
        ExecuteMsg::ApproveWork { job_id } => Ok(execute_approve_work(deps, env, info, job_id)?),
    }
}

pub mod execute {
    use cosmwasm_std::{Addr, CosmosMsg, StdError, Uint128, WasmMsg};
    use cw20::{AllowanceResponse, BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

    use crate::state::{
        Job, JobStatus, Profile, ProfileType, SocialLink, JOBS, PROFILES, PROPOSALS,
    };

    use super::*;

    pub fn execute_create_profile(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        profile_type: ProfileType,
        name: String,
        bio: String,
        skills: Vec<String>,
        hourly_rate: Uint128,
        social_links: Vec<SocialLink>,
    ) -> StdResult<Response> {
        // Check if profile already exists
        if PROFILES
            .may_load(deps.storage, info.sender.as_ref())?
            .is_some()
        {
            return Err(StdError::generic_err("Profile already exists"));
        }

        let profile = Profile {
            address: info.sender.to_string(),
            profile_type,
            name,
            bio,
            skills,
            hourly_rate,
            rating: None,
            completed_jobs: 0,
            created_at: env.block.time.seconds(),
            social_links,
            verified: false,
        };

        PROFILES.save(deps.storage, info.sender.as_ref(), &profile)?;

        Ok(Response::new()
            .add_attribute("action", "create_profile")
            .add_attribute("address", info.sender))
    }

    pub fn execute_update_profile(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        name: Option<String>,
        bio: Option<String>,
        skills: Option<Vec<String>>,
        hourly_rate: Option<Uint128>,
        social_links: Option<Vec<SocialLink>>,
    ) -> StdResult<Response> {
        // Check if profile exists
        if PROFILES
            .may_load(deps.storage, info.sender.as_ref())?
            .is_none()
        {
            return Err(StdError::generic_err("Profile does not exists"));
        }

        PROFILES.update(
            deps.storage,
            info.sender.as_ref(),
            |profile| match profile {
                Some(mut profile) => {
                    if let Some(name) = name {
                        profile.name = name;
                    }
                    if let Some(bio) = bio {
                        profile.bio = bio;
                    }
                    if let Some(skills) = skills {
                        profile.skills = skills;
                    }
                    if let Some(hourly_rate) = hourly_rate {
                        profile.hourly_rate = hourly_rate;
                    }
                    if let Some(social_links) = social_links {
                        profile.social_links = social_links;
                    }
                    Ok(profile)
                }
                None => Err(StdError::generic_err("Profile does not exist")),
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "update_profile")
            .add_attribute("address", info.sender))
    }

    pub fn execute_post_job(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        title: String,
        description: String,
        budget: Uint128,
        deadline: u64,
        deliverables: Vec<String>,
    ) -> StdResult<Response> {
        // Check if profile exists
        let profile = PROFILES.may_load(deps.storage, info.sender.as_ref())?;
        if profile.is_none() {
            return Err(StdError::generic_err("Profile does not exists"));
        }

        if profile.unwrap().profile_type != ProfileType::Client {
            return Err(StdError::generic_err("Only clients can post jobs"));
        }

        // Validate deadline
        if env.block.time.seconds() > u64::MAX - deadline {
            return Err(StdError::generic_err("Deadline too far in the future"));
        }

        // Calculate future deadline safely
        let future_deadline = env
            .block
            .time
            .seconds()
            .checked_add(deadline)
            .ok_or_else(|| {
                StdError::generic_err("Failed to calculate future deadline due to overflow")
            })?;

        let state = STATE.load(deps.storage)?;

        // Add allowance check
        let allowance: AllowanceResponse = deps.querier.query_wasm_smart(
            state.token_address.clone(),
            &Cw20QueryMsg::Allowance {
                owner: info.sender.to_string(),
                spender: env.contract.address.to_string(),
            },
        )?;

        if allowance.allowance < budget {
            return Err(StdError::generic_err("Insufficient allowance"));
        }

        // Check token balance
        let balance: BalanceResponse = deps.querier.query_wasm_smart(
            state.token_address.clone(),
            &Cw20QueryMsg::Balance {
                address: info.sender.to_string(),
            },
        )?;

        if balance.balance < budget {
            return Err(StdError::generic_err("Insufficient token balance"));
        }

        // Generate unique job ID (using timestamp + client address)
        let job_id = format!(
            "job_{}_{}",
            env.block.time.seconds(),
            info.sender.to_string()
        );

        let job = Job {
            id: job_id.clone(),
            client: info.sender.clone(),
            freelancer: None,
            title,
            description,
            budget,
            status: JobStatus::Open,
            deadline: future_deadline,
            deliverables,
            submission: None,
        };

        let mut messages: Vec<CosmosMsg> = vec![];

        // Transfer ATOM tokens to escrow account
        let transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: state.escrow_address.to_string(),
            amount: budget,
        };

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_address.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: vec![],
        }));

        JOBS.save(deps.storage, &job_id, &job)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "post_job")
            .add_attribute("job_id", job_id)
            .add_attribute("client", info.sender.to_string()))
    }

    pub fn execute_apply_for_job(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        job_id: String,
        proposal: String,
    ) -> StdResult<Response> {
        // Check if profile exists
        let profile = PROFILES.may_load(deps.storage, info.sender.as_ref())?;
        if profile.is_none() {
            return Err(StdError::generic_err("Profile does not exists"));
        }

        if profile.unwrap().profile_type != ProfileType::Freelancer {
            return Err(StdError::generic_err("Only freelancers can apply for jobs"));
        }

        let job = JOBS.load(deps.storage, &job_id)?;

        if job.status != JobStatus::Open {
            return Err(StdError::generic_err("Job is not open for proposals"));
        }

        // Save the proposal
        PROPOSALS.save(deps.storage, (&job_id, &info.sender), &proposal)?;

        Ok(Response::new()
            .add_attribute("action", "apply_for_job")
            .add_attribute("job_id", job_id)
            .add_attribute("freelancer", info.sender.to_string()))
    }

    pub fn execute_accept_proposal(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        job_id: String,
        freelancer: Addr,
    ) -> StdResult<Response> {
        // Check if profile exists
        let profile = PROFILES.may_load(deps.storage, info.sender.as_ref())?;
        if profile.is_none() {
            return Err(StdError::generic_err("Profile does not exists"));
        }

        if profile.unwrap().profile_type != ProfileType::Client {
            return Err(StdError::generic_err("Only clients can accept jobs"));
        }

        let mut job = JOBS.load(deps.storage, &job_id)?;

        // Verify sender is the client
        if info.sender != job.client {
            return Err(StdError::generic_err(
                "Only the client can accept proposals",
            ));
        }

        // Verify job is still open
        if job.status != JobStatus::Open {
            return Err(StdError::generic_err("Job is not open"));
        }

        // Update job with accepted freelancer
        job.freelancer = Some(freelancer.clone());
        job.status = JobStatus::InProgress;
        JOBS.save(deps.storage, &job_id, &job)?;

        Ok(Response::new()
            .add_attribute("action", "accept_proposal")
            .add_attribute("job_id", job_id)
            .add_attribute("freelancer", freelancer))
    }

    pub fn execute_submit_work(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        job_id: String,
        submission: String,
    ) -> StdResult<Response> {
        // Check if profile exists
        let profile = PROFILES.may_load(deps.storage, info.sender.as_ref())?;
        if profile.is_none() {
            return Err(StdError::generic_err("Profile does not exists"));
        }

        if profile.unwrap().profile_type != ProfileType::Client {
            return Err(StdError::generic_err("Only freelancer can accept jobs"));
        }

        let mut job = JOBS.load(deps.storage, &job_id)?;

        // Verify sender is the assigned freelancer
        if job.freelancer != Some(info.sender) {
            return Err(StdError::generic_err(
                "Only the assigned freelancer can submit work",
            ));
        }

        // Verify job is in progress
        if job.status != JobStatus::InProgress {
            return Err(StdError::generic_err("Job is not in progress"));
        }

        // Update job with submission
        job.submission = Some(submission);
        job.status = JobStatus::UnderReview;
        JOBS.save(deps.storage, &job_id, &job)?;

        Ok(Response::new()
            .add_attribute("action", "submit_work")
            .add_attribute("job_id", job_id))
    }

    pub fn execute_approve_work(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        job_id: String,
    ) -> StdResult<Response> {
        // Check if profile exists
        let profile = PROFILES.may_load(deps.storage, info.sender.as_ref())?;
        if profile.is_none() {
            return Err(StdError::generic_err("Profile does not exists"));
        }

        if profile.unwrap().profile_type != ProfileType::Client {
            return Err(StdError::generic_err("Only clients can approve jobs"));
        }

        let mut job = JOBS.load(deps.storage, &job_id)?;
        let state = STATE.load(deps.storage)?;

        // Verify sender is the client
        if info.sender != job.client {
            return Err(StdError::generic_err("Only the client can approve work"));
        }

        // Verify job is under review
        if job.status != JobStatus::UnderReview {
            return Err(StdError::generic_err("Job is not under review"));
        }

        // Calculate platform fee
        let platform_fee = job
            .budget
            .multiply_ratio(state.platform_fee.to_uint_ceil(), 10000u64);
        let freelancer_payment = job.budget - platform_fee;

        // Update job status
        job.status = JobStatus::Completed;
        JOBS.save(deps.storage, &job_id, &job)?;

        // Create payment messages
        let mut messages: Vec<CosmosMsg> = vec![];

        // Payment to freelancer
        if let Some(freelancer) = &job.freelancer {
            let transfer_msg = Cw20ExecuteMsg::Transfer {
                recipient: freelancer.to_string(),
                amount: freelancer_payment,
            };

            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.token_address.to_string(),
                msg: to_json_binary(&transfer_msg)?,
                funds: vec![],
            }));
        }

        // Platform fee
        let transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: state.escrow_address.to_string(),
            amount: platform_fee,
        };

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_address.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: vec![],
        }));

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "approve_work")
            .add_attribute("job_id", job_id)
            .add_attribute("payment_amount", freelancer_payment.to_string())
            .add_attribute("platform_fee", platform_fee.to_string()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProfile { address } => to_json_binary(&query_profile(deps, address)?),
        QueryMsg::ListProfiles {
            profile_type,
            start_after,
            limit,
        } => to_json_binary(&query_list_profiles(
            deps,
            profile_type,
            start_after,
            limit,
        )?),
        QueryMsg::SearchFreelancers {
            skills,
            min_rating,
            max_hourly_rate,
            start_after,
            limit,
        } => to_json_binary(&query_search_freelancers(
            deps,
            skills,
            min_rating,
            max_hourly_rate,
            start_after,
            limit,
        )?),
        QueryMsg::GetJob { job_id } => to_json_binary(&query_job(deps, job_id)?),
        QueryMsg::ListJobs {
            status,
            start_after,
            limit,
        } => to_json_binary(&query_list_jobs(deps, status, start_after, limit)?),
        QueryMsg::GetFreelancerJobs { address, status } => {
            to_json_binary(&query_freelancer_jobs(deps, address, status)?)
        }
        QueryMsg::GetClientJobs { address, status } => {
            to_json_binary(&query_client_jobs(deps, address, status)?)
        }
    }
}

pub mod query {
    use cosmwasm_std::{Addr, Order, Uint128};
    use cw_storage_plus::Bound;

    use crate::{
        msg::{
            GetJobResponse, GetListJobsResponse, GetListProfilesResponse, GetProfileResponse,
            GetSearchFreelancersResponse,
        },
        state::{Job, JobStatus, Profile, ProfileType, JOBS, PROFILES},
    };

    use super::*;

    pub fn query_profile(deps: Deps, address: String) -> StdResult<GetProfileResponse> {
        let profile = PROFILES.load(deps.storage, &address)?;

        Ok(GetProfileResponse { profile })
    }

    pub fn query_list_profiles(
        deps: Deps,
        profile_type: Option<ProfileType>,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<GetListProfilesResponse> {
        let limit = limit.unwrap_or(30) as usize;
        let start = start_after.map(|s| s).unwrap();

        let res: StdResult<Vec<Profile>> = PROFILES
            .range(
                deps.storage,
                Some(Bound::inclusive(&*start)),
                None,
                Order::Ascending,
            )
            .filter(|r| {
                if let Ok((_, profile)) = r {
                    if let Some(pt) = &profile_type {
                        return profile.profile_type == *pt;
                    }
                    true
                } else {
                    false
                }
            })
            .take(limit)
            .map(|item| item.map(|(_, profile)| profile))
            .collect();

        let profiles = res.unwrap();

        Ok(GetListProfilesResponse { profiles })
    }

    pub fn query_search_freelancers(
        deps: Deps,
        skills: Vec<String>,
        min_rating: Option<f64>,
        max_hourly_rate: Option<Uint128>,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<GetSearchFreelancersResponse> {
        let limit = limit.unwrap_or(30) as usize;
        let start = start_after.map(|s| s).unwrap();

        let res: StdResult<Vec<Profile>> = PROFILES
            .range(
                deps.storage,
                Some(Bound::inclusive(&*start)),
                None,
                Order::Ascending,
            )
            .filter(|r| {
                if let Ok((_, profile)) = r {
                    // Filter by skills
                    if !skills.is_empty()
                        && !skills.iter().any(|skill| profile.skills.contains(skill))
                    {
                        return false;
                    }

                    // Filter by rating
                    if let Some(min_rating) = min_rating {
                        if let Some(rating) = &profile.rating {
                            if rating.average < min_rating {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }

                    // Filter by hourly rate
                    if let Some(max_rate) = max_hourly_rate {
                        if profile.hourly_rate > max_rate {
                            return false;
                        }
                    }

                    true
                } else {
                    false
                }
            })
            .take(limit)
            .map(|item| item.map(|(_, profile)| profile))
            .collect();

        let profiles = res.unwrap();

        Ok(GetSearchFreelancersResponse { profiles })
    }

    pub fn query_job(deps: Deps, job_id: String) -> StdResult<GetJobResponse> {
        let job = JOBS.load(deps.storage, &job_id)?;

        return Ok(GetJobResponse { job });
    }

    pub fn query_list_jobs(
        deps: Deps,
        status: Option<JobStatus>,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<GetListJobsResponse> {
        let limit = limit.unwrap_or(30) as usize;
        let start = start_after.map(|s| s).unwrap();

        let res: StdResult<Vec<Job>> = JOBS
            .range(
                deps.storage,
                Some(Bound::inclusive(&*start)),
                None,
                Order::Ascending,
            )
            .filter(|r| {
                if let Ok((_, job)) = r {
                    if let Some(s) = &status {
                        return job.status == *s;
                    }
                    true
                } else {
                    false
                }
            })
            .take(limit)
            .map(|item| item.map(|(_, job)| job))
            .collect();

        let jobs = res.unwrap();

        return Ok(GetListJobsResponse { jobs });
    }

    pub fn query_freelancer_jobs(
        deps: Deps,
        address: Addr,
        status: Option<JobStatus>,
    ) -> StdResult<Vec<Job>> {
        let jobs: StdResult<Vec<Job>> = JOBS
            .range(deps.storage, None, None, Order::Ascending)
            .filter(|r| {
                if let Ok((_, job)) = r {
                    if let Some(freelancer) = &job.freelancer {
                        if freelancer != &address {
                            return false;
                        }
                        if let Some(s) = &status {
                            return job.status == *s;
                        }
                        return true;
                    }
                    false
                } else {
                    false
                }
            })
            .map(|item| item.map(|(_, job)| job))
            .collect();

        jobs
    }

    pub fn query_client_jobs(
        deps: Deps,
        address: Addr,
        status: Option<JobStatus>,
    ) -> StdResult<Vec<Job>> {
        let jobs: StdResult<Vec<Job>> = JOBS
            .range(deps.storage, None, None, Order::Ascending)
            .filter(|r| {
                if let Ok((_, job)) = r {
                    if job.client != address {
                        return false;
                    }
                    if let Some(s) = &status {
                        return job.status == *s;
                    }
                    true
                } else {
                    false
                }
            })
            .map(|item| item.map(|(_, job)| job))
            .collect();

        jobs
    }
}
