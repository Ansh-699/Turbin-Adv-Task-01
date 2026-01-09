use anchor_lang::prelude::*;

declare_id!("Gi9ZPReh1pPsvpukkvuRMbboJtcpPh4ryqnFz4tkhLbJ");

#[program]
pub mod limited_claim {
    use super::*;

    pub fn initialize_counter(
        ctx: Context<InitializeCounter>,
        capacity: u64,
        start_time: i64,
    ) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.admin = *ctx.accounts.admin.key;
        counter.capacity = capacity;
        counter.start_time = start_time;
        counter.remaining = capacity;
        counter.bump = ctx.bumps.counter;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;

        require!(counter.remaining > 0, CustomError::SoldOut);
        require!(
            Clock::get()?.unix_timestamp >= counter.start_time,
            CustomError::NotStarted
        );

        counter.remaining = counter
            .remaining
            .checked_sub(1)
            .ok_or(CustomError::SoldOut)?;

        let receipt = &mut ctx.accounts.receipt;
        receipt.claimer = *ctx.accounts.claimer.key;
        receipt.claimed_at = Clock::get()?.unix_timestamp;
        receipt.bump = ctx.bumps.receipt;

        emit!(ClaimEvent {
            claimer: receipt.claimer,
            counter: *counter.to_account_info().key,
            remaining: counter.remaining,
        });

        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        let receipt = &ctx.accounts.receipt;

        require_keys_eq!(
            receipt.claimer,
            *ctx.accounts.claimer.key,
            CustomError::Unauthorized
        );

        require!(
            counter.remaining < counter.capacity,
            CustomError::CounterAtCapacity
        );

        counter.remaining = counter
            .remaining
            .checked_add(1)
            .ok_or(CustomError::CounterOverflow)?;

        emit!(CancelEvent {
            claimer: receipt.claimer,
            counter: *counter.to_account_info().key,
            remaining: counter.remaining,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(capacity: u64, start_time: i64)]
pub struct InitializeCounter<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Counter::SIZE,
        seeds = [b"counter", admin.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"counter", counter.admin.as_ref()],
        bump = counter.bump,
    )]
    pub counter: Account<'info, Counter>,

    #[account(
        init,
        payer = claimer,
        space = 8 + ClaimReceipt::SIZE,
        seeds = [b"receipt", counter.key().as_ref(), claimer.key().as_ref()],
        bump
    )]
    pub receipt: Account<'info, ClaimReceipt>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(
        mut,
        seeds = [b"counter", counter.admin.as_ref()],
        bump = counter.bump,
    )]
    pub counter: Account<'info, Counter>,

    #[account(
        mut,
        seeds = [b"receipt", counter.key().as_ref(), claimer.key().as_ref()],
        bump = receipt.bump,
        close = claimer
    )]
    pub receipt: Account<'info, ClaimReceipt>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Counter {
    pub admin: Pubkey,
    pub capacity: u64,
    pub start_time: i64,
    pub remaining: u64,
    pub bump: u8,
}

impl Counter {
    // 32 (admin) + 8 (capacity) + 8 (start_time) + 8 (remaining) + 1 (bump) = 57
    pub const SIZE: usize = 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct ClaimReceipt {
    pub claimer: Pubkey,
    pub claimed_at: i64,
    pub bump: u8,
}

impl ClaimReceipt {
    // 32 (claimer) + 8 (claimed_at) + 1 (bump) = 41
    pub const SIZE: usize = 32 + 8 + 1;
}

//errors and events

#[event]
pub struct ClaimEvent {
    pub claimer: Pubkey,
    pub counter: Pubkey,
    pub remaining: u64,
}

#[event]
pub struct CancelEvent {
    pub claimer: Pubkey,
    pub counter: Pubkey,
    pub remaining: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("Sold out")]
    SoldOut,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Counter at capacity")]
    CounterAtCapacity,
    #[msg("Counter overflow")]
    CounterOverflow,
    #[msg("Claiming has not started yet")]
    NotStarted,
}
