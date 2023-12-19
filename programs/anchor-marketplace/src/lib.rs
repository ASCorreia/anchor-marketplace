mod contexts;
use contexts::*;

pub mod state;
pub use state::*;

use anchor_lang::prelude::*;

declare_id!("mktYdagPAAnuHigRD62zLpHshZqx7vpKHjN3fN6MPjy");

#[program]
pub mod anchor_marketplace {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, fee: u16) -> Result<()> {
        ctx.accounts.init(name, fee, &ctx.bumps)
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, &ctx.bumps)?;
        ctx.accounts.deposit_nft()
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.withdraw_nft()
    }

    pub fn purchase(ctx: Context<Purchase>) -> Result<()> {
        ctx.accounts.send_sol()?;
        ctx.accounts.send_nft()?;
        ctx.accounts.mint_rewards()?;
        ctx.accounts.close_mint_ata()
    }
}
