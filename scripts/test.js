import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";
import dotenv from "dotenv"

dotenv.config()

const rpcEndpoint = "https://rpc-palvus.pion-1.ntrn.tech";
const freelancer_mnemonic = process.env.FREELANCER_MNEMONIC;
const client_mnemonic = process.env.CLIENT_MNEMONIC;
const upworkWasmFilePath = "./artifacts/upwork.wasm";
const tokenWasmFilePath = "./artifacts/first_token_cw20contract.wasm";

const test = async () => {
    const freelancer_wallet = await DirectSecp256k1HdWallet.fromMnemonic(freelancer_mnemonic, {
        prefix: "neutron",
    });

    const [firstAccount] = await freelancer_wallet.getAccounts();

    const client_wallet = await DirectSecp256k1HdWallet.fromMnemonic(client_mnemonic, {
        prefix: "neutron",
    });

    const [secondAccount] = await client_wallet.getAccounts();

    const freelancerClient = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        freelancer_wallet,
        {
            gasPrice: GasPrice.fromString("0.025untrn"),
        }
    );

    const clientClient = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        client_wallet,
        {
            gasPrice: GasPrice.fromString("0.025untrn"),
        }
    );

    // Upload token contract
    const tokenWasmCode = readFileSync(tokenWasmFilePath);
    const tokenUploadReceipt = await freelancerClient.upload(firstAccount.address, tokenWasmCode, "auto");
    console.log("Token upload successful, code ID:", tokenUploadReceipt.codeId);

    const initMsg = {
        name: "Test ATOM",
        symbol: "tATOM",
        decimals: 6,
        initial_balances: [
            {
                "address": firstAccount.address,
                "amount": "10000000000000"
            },
            {
                "address": secondAccount.address,
                "amount": "10000000000000"
            }
        ]
    };

    // Deploy token contract
    const instantiateReceipt = await freelancerClient.instantiate(firstAccount.address, tokenUploadReceipt.codeId, initMsg, "CW Token", "auto");
    console.log("Token Contract instantiated at:", instantiateReceipt.contractAddress);

    // Upload up work contract
    const upworkWasmCode = readFileSync(upworkWasmFilePath);
    const upworkUploadReceipt = await freelancerClient.upload(firstAccount.address, upworkWasmCode, "auto");
    console.log("Upwork code upload successful, code ID:", upworkUploadReceipt.codeId);

    // Deploy up work contract
    const upworkInitMsg = {
        admin: firstAccount.address,
        platform_fee: 1,
        escrow_address: firstAccount.address,
        token_address:  instantiateReceipt.contractAddress,
    };
    const upworkInstantiateReceipt = await freelancerClient.instantiate(firstAccount.address, upworkUploadReceipt.codeId, upworkInitMsg, "Atom Work", "auto");
    console.log("Upwork contract instantiated at:", upworkInstantiateReceipt.contractAddress);

    //--------------------------------- Calls
    const freelancer = firstAccount.address;
    const client = secondAccount.address;

    const createFreelancerProfileMsg = {
        create_profile: {
            profile_type: "Freelancer",
            name: "freelancer name",
            bio: "freelancer bio",
            skills: ["skill1", "skill2", "skill3"],
            hourly_rate: "1",
            social_links: [
                {
                    platform: "X",
                    url: "url",
                }
            ],
        }
        
    }
    const upworkCreateFreelancerProfile = await freelancerClient.execute(freelancer,upworkInstantiateReceipt.contractAddress, createFreelancerProfileMsg, "auto");
    console.log("Created freelancer profile successfully. Transaction Hash: ", upworkCreateFreelancerProfile.transactionHash);

    const createClientProfileMsg = {
        create_profile: {
            profile_type: "Client",
            name: "client name",
            bio: "client bio",
            skills: ["skill1", "skill2", "skill3"],
            hourly_rate: "2",
            social_links: [
                {
                    platform: "X",
                    url: "url",
                }
            ],
        }
    }
    const upworkCreateClientProfile = await clientClient.execute(client, upworkInstantiateReceipt.contractAddress, createClientProfileMsg, "auto");
    console.log("Created client profile successfully. Transaction Hash: ", upworkCreateClientProfile.transactionHash);

    const updateClientProfileMsg = {
        update_profile: {
            name: "client name",
            bio: "new client bio",
            skills: ["skill1", "skill2", "skill3"],
            hourly_rate: "3",
            social_links: [
                {
                    platform: "Youtube",
                    url: "yt_url",
                }
            ],
        }
    }
    const upworkUpdateClientProfile = await clientClient.execute(client, upworkInstantiateReceipt.contractAddress, updateClientProfileMsg, "auto");
    console.log("Updated client profile successfully. Transaction Hash: ", upworkUpdateClientProfile.transactionHash);

    // Increase allowance before posting job
    const increaseAllowanceBeforeJobPostMsg = {
        increase_allowance: {
            spender:  upworkInstantiateReceipt.contractAddress,
            amount: "0.1",
        }
    }
    const increaseAllowanceBeforeJobPost = await clientClient.execute(client,  instantiateReceipt.contractAddress, 
        increaseAllowanceBeforeJobPostMsg, "auto");
    console.log("Approve spending successfully. Transaction Hash: ", increaseAllowanceBeforeJobPost.transactionHash);

    const postJobMsg = {
        post_job: {
            title: "title",
            description: "description",
            budget: "0.1",
            deadline: 10, // seconds
            deliverables: ["deliverable1", "deliverable2"],
        }        
    }
    const postJob = await clientClient.execute(client, upworkInstantiateReceipt.contractAddress, postJobMsg, "auto");
    console.log("Posted job successfully. Transaction Hash: ", postJob.transactionHash);

    const getJobsQuery = {
        list_jobs: {
           start_after: "0"
        }
    }
    const res = await freelancerClient.queryContractSmart( 
        "neutron1tqxpa00c7vdqq5vnl8djx74a24qkqyzmqck9059qxrp9te6ap0fs589pua", 
        getJobsQuery)

    const job_id = res.jobs[0].id;
    console.log("Jobs Id: ", job_id);

    const applyJobMsg = {
        apply_for_job: {
            job_id: job_id,
            proposal: "my proposal",
        }
    }
    const applyJob = await freelancerClient.execute(freelancer, upworkInstantiateReceipt.contractAddress, applyJobMsg, "auto");
    console.log("Applied for job successfully. Transaction Hash: ", applyJob.transactionHash);

    const acceptProposalMsg = {
        accept_proposal: {
            job_id: job_id,
            freelancer: freelancer,
        }
    }
    const acceptProposal = await clientClient.execute(client, upworkInstantiateReceipt.contractAddress, acceptProposalMsg, "auto");
    console.log("Accepted proposal successfully. Transaction Hash: ", acceptProposal.transactionHash);

    const submitWorkMsg = {
        submit_work: {
            job_id: job_id,
            submission: "my submission",
        }
    }
    const submitWork = await freelancerClient.execute(freelancer, upworkInstantiateReceipt.contractAddress, submitWorkMsg, "auto");
    console.log("Submitted work successfully. Transaction Hash: ", submitWork.transactionHash);

    const approveWorkMsg = {
        approve_work: {
            job_id: job_id,
        }
    }
    const approveWork = await clientClient.execute(freelancer, upworkInstantiateReceipt.contractAddress, approveWorkMsg, "auto");
    console.log("Approved job successfully. Transaction Hash: ", approveWork.transactionHash);


    //---------------------- Queries
    const getProfileQuery = { 
        get_profile: {
            address: freelancer
        }   
    }
    const getProfileRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, getProfileQuery)
    console.log("Profile: ", getProfileRes);

    const listProfileQuery = { 
        list_profiles: {
            start_after: ""
        }        
    }
    const listProfileRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, listProfileQuery)
    console.log("Profile: ", listProfileRes);

    const searchFreelancersQuery = { 
        search_freelancers: {
            skills: "skill1",
            start_after: ""
        }
    }
    const SearchFreelancersRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, searchFreelancersQuery)
    console.log("Profile: ", SearchFreelancersRes);

    const getJobQuery = { 
        get_job: {
            job_id: job_id
        } 
    }
    const getJobRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, getJobQuery)
    console.log("Profile: ", getJobRes);

    const listJobsQuery = {
        list_jobs: {
            start_after: ""
        }
    }
    const listJobsRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, listJobsQuery)
    console.log("Jobs: ", listJobsRes);

    const getFreelancerJobsQuery = {
        get_freelancer_jobs: {
            address: freelancer
        }
    }
    const getFreelancerJobsRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, getFreelancerJobsQuery)
    console.log("Jobs: ", getFreelancerJobsRes);

    const getClientJobsQuery = {
        get_client_jobs: {
            address: client
        }
    }
    const getClientJobsRes = await freelancerClient.queryContractSmart(upworkInstantiateReceipt.contractAddress, getClientJobsQuery)
    console.log("Jobs: ", getClientJobsRes);
}

test().catch(console.error);