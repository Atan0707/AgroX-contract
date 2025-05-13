use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token};
use std::collections::BTreeMap;

declare_id!("Go8PqyzmSG4LfAoo1PqGRbgctXynnRYAVa9RQx9aZRHC");

#[program]
pub mod agrox_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let system_state = &mut ctx.accounts.system_state;
        system_state.authority = ctx.accounts.authority.key();
        system_state.machine_count = 0;
        system_state.total_data_uploads = 0;
        system_state.data_request_count = 0;

        msg!("AgroX system initialized by: {}", system_state.authority);
        Ok(())
    }

    pub fn register_machine(ctx: Context<RegisterMachine>, machine_id: String) -> Result<()> {
        // Validate machine ID isn't already used
        let machines = &ctx.accounts.system_state.machines;
        require!(!machines.contains_key(&machine_id), ErrorCode::MachineIdAlreadyExists);

        // Create and initialize the machine account
        let machine = &mut ctx.accounts.machine;
        machine.owner = ctx.accounts.user.key();
        machine.machine_id = machine_id.clone();
        machine.is_active = false;
        machine.data_count = 0;
        machine.image_count = 0;
        machine.rewards_earned = 0;
        machine.last_data_timestamp = 0;
        machine.last_image_timestamp = 0;
        machine.data_used_count = 0;

        // Add machine to system state
        let system_state = &mut ctx.accounts.system_state;
        system_state.machines.insert(machine_id.clone(), ctx.accounts.machine.key());
        system_state.machine_count += 1;

        msg!("Machine registered: {}", machine_id);
        Ok(())
    }

    pub fn start_machine(ctx: Context<ControlMachine>) -> Result<()> {
        let machine = &mut ctx.accounts.machine;
        
        // Only the machine owner can start it
        require!(machine.owner == ctx.accounts.user.key(), ErrorCode::Unauthorized);
        
        // Set machine as active
        machine.is_active = true;
        
        msg!("Machine started: {}", machine.machine_id);
        Ok(())
    }

    pub fn stop_machine(ctx: Context<ControlMachine>) -> Result<()> {
        let machine = &mut ctx.accounts.machine;
        
        // Only the machine owner can stop it
        require!(machine.owner == ctx.accounts.user.key(), ErrorCode::Unauthorized);
        
        // Set machine as inactive
        machine.is_active = false;
        
        msg!("Machine stopped: {}", machine.machine_id);
        Ok(())
    }

    pub fn upload_data(
        ctx: Context<UploadData>,
        temperature: f64,
        humidity: f64,
        image_url: Option<String>,
    ) -> Result<()> {
        let machine = &mut ctx.accounts.machine;
        let system_state = &mut ctx.accounts.system_state;
        let clock = Clock::get()?;
        
        // Only allow uploads if machine is active
        require!(machine.is_active, ErrorCode::MachineNotActive);
        
        // Create and initialize the data account
        let data = &mut ctx.accounts.data;
        data.machine = machine.key();
        data.timestamp = clock.unix_timestamp;
        data.temperature = temperature;
        data.humidity = humidity;
        data.image_url = image_url.clone();
        data.used_count = 0;
        
        // Update machine and system state
        machine.data_count += 1;
        machine.last_data_timestamp = clock.unix_timestamp;
        system_state.total_data_uploads += 1;
        
        // Check if this upload includes an image
        if image_url.is_some() {
            machine.image_count += 1;
            machine.last_image_timestamp = clock.unix_timestamp;
            
            // Additional reward for including an image
            machine.rewards_earned += 10; // 10 tokens per image
        }
        
        // Base reward for sensor data
        machine.rewards_earned += 1; // 1 token per data upload
        
        msg!("Data uploaded from machine: {}", machine.machine_id);
        Ok(())
    }

    pub fn use_data(ctx: Context<UseData>) -> Result<()> {
        let data = &mut ctx.accounts.data;
        let machine = &mut ctx.accounts.machine;
        let user = &ctx.accounts.user;
        let system_state = &mut ctx.accounts.system_state;
        
        // Update usage count
        data.used_count += 1;
        machine.data_used_count += 1;
        system_state.data_request_count += 1;
        
        // Calculate and apply rewards to machine owner
        let reward_amount = 2; // 2 tokens per data usage
        machine.rewards_earned += reward_amount;
        
        msg!("Data used by: {}", user.key());
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let machine = &mut ctx.accounts.machine;
        
        // Only owner can claim rewards
        require!(machine.owner == ctx.accounts.user.key(), ErrorCode::Unauthorized);
        
        // Check there are rewards to claim
        let rewards = machine.rewards_earned;
        require!(rewards > 0, ErrorCode::NoRewardsAvailable);
        
        // Reset rewards in the machine account
        machine.rewards_earned = 0;
        
        // In a real implementation, you would transfer tokens here
        // For now we just log the claim
        msg!("Rewards claimed: {} tokens for machine: {}", rewards, machine.machine_id);
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = SystemState::SPACE
    )]
    pub system_state: Account<'info, SystemState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterMachine<'info> {
    #[account(mut)]
    pub system_state: Account<'info, SystemState>,
    
    #[account(
        init,
        payer = user,
        space = Machine::SPACE
    )]
    pub machine: Account<'info, Machine>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ControlMachine<'info> {
    #[account(mut)]
    pub machine: Account<'info, Machine>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UploadData<'info> {
    #[account(mut)]
    pub system_state: Account<'info, SystemState>,
    
    #[account(mut)]
    pub machine: Account<'info, Machine>,
    
    #[account(
        init,
        payer = uploader,
        space = IoTData::SPACE
    )]
    pub data: Account<'info, IoTData>,
    
    #[account(mut)]
    pub uploader: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UseData<'info> {
    #[account(mut)]
    pub system_state: Account<'info, SystemState>,
    
    #[account(mut)]
    pub machine: Account<'info, Machine>,
    
    #[account(mut)]
    pub data: Account<'info, IoTData>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub machine: Account<'info, Machine>,
    
    pub user: Signer<'info>,
}

#[account]
pub struct SystemState {
    pub authority: Pubkey,
    pub machine_count: u64,
    pub total_data_uploads: u64,
    pub data_request_count: u64,
    pub machines: BTreeMap<String, Pubkey>,
}

impl SystemState {
    pub const SPACE: usize = 8 + // discriminator
                            32 + // authority
                            8 + // machine_count
                            8 + // total_data_uploads
                            8 + // data_request_count
                            500; // machines map (approx space)
}

#[account]
pub struct Machine {
    pub owner: Pubkey,
    pub machine_id: String,
    pub is_active: bool,
    pub data_count: u64,
    pub image_count: u64,
    pub rewards_earned: u64,
    pub last_data_timestamp: i64,
    pub last_image_timestamp: i64,
    pub data_used_count: u64,
}

impl Machine {
    pub const SPACE: usize = 8 + // discriminator
                            32 + // owner
                            36 + // machine_id (max 32 chars + 4 bytes for length)
                            1 + // is_active
                            8 + // data_count
                            8 + // image_count
                            8 + // rewards_earned
                            8 + // last_data_timestamp
                            8 + // last_image_timestamp
                            8; // data_used_count
}

#[account]
pub struct IoTData {
    pub machine: Pubkey,
    pub timestamp: i64,
    pub temperature: f64,
    pub humidity: f64,
    pub image_url: Option<String>,
    pub used_count: u64,
}

impl IoTData {
    pub const SPACE: usize = 8 + // discriminator
                            32 + // machine
                            8 + // timestamp
                            8 + // temperature
                            8 + // humidity
                            (1 + 100) + // Option<String> (1 for is_some flag + max 100 bytes for URL)
                            8; // used_count
}

#[error_code]
pub enum ErrorCode {
    #[msg("Machine ID already exists")]
    MachineIdAlreadyExists,
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Machine is not active")]
    MachineNotActive,
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
}
