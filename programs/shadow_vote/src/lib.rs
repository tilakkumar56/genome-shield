use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;

const COMP_DEF_OFFSET_CAST_VOTE: u32 = comp_def_offset("cast_vote");

declare_id!("H6NrSVGXBpp5jdrEAaLHuWLsmPUhMt9yK2uujQotNmKU");

#[arcium_program]
pub mod shadow_vote {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.program_state;
        state.authority = ctx.accounts.authority.key();
        state.total_proposals = 0;
        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        num_options: u8,
        option_labels: Vec<String>,
        voting_ends_at: i64,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        proposal.creator = ctx.accounts.authority.key();
        proposal.title = title;
        proposal.description = description;
        proposal.num_options = num_options;
        let mut labels = [String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new()];
        for (i, l) in option_labels.iter().enumerate() {
            if i < 8 { labels[i] = l.clone(); }
        }
        proposal.option_labels = labels;
        proposal.voting_ends_at = voting_ends_at;
        proposal.finalized = false;
        proposal.results = [0u64; 8];
        proposal.total_votes = 0;
        let state = &mut ctx.accounts.program_state;
        proposal.proposal_id = state.total_proposals;
        state.total_proposals += 1;
        Ok(())
    }

    pub fn init_cast_vote_comp_def(ctx: Context<InitCastVoteCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn cast_vote(
        ctx: Context<CastVote>,
        computation_offset: u64,
        ct_option_idx: [u8; 32],
        ct_weight: [u8; 32],
        ct_num_options: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        let args = ArgBuilder::new()
            .x25519_pubkey(pub_key)
            .plaintext_u128(nonce)
            .encrypted_u8(ct_option_idx)
            .encrypted_u128(ct_weight)
            .encrypted_u8(ct_num_options)
            .build();

        let vote_record_key = ctx.accounts.vote_record.key();
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![CastVoteCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount { pubkey: vote_record_key, is_writable: true }],
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "cast_vote")]
    pub fn cast_vote_callback(
        ctx: Context<CastVoteCallback>,
        output: SignedComputationOutputs<CastVoteOutput>,
    ) -> Result<()> {
        let _o = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(CastVoteOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let record = &mut ctx.accounts.vote_record;
        record.voted = true;
        emit!(VoteEvent { voter: record.voter });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + ProgramState::INIT_SPACE, seeds = [b"program_state"], bump)]
    pub program_state: Account<'info, ProgramState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + Proposal::INIT_SPACE, seeds = [b"proposal", authority.key().as_ref(), &program_state.total_proposals.to_le_bytes()], bump)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut, seeds = [b"program_state"], bump)]
    pub program_state: Account<'info, ProgramState>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("cast_vote", payer)]
#[derive(Accounts)]
pub struct InitCastVoteCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: address_lookup_table
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: lut_program
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("cast_vote", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CAST_VOTE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(init, payer = payer, space = 8 + VoteRecord::INIT_SPACE, seeds = [b"vote_record", computation_offset.to_le_bytes().as_ref()], bump)]
    pub vote_record: Account<'info, VoteRecord>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("cast_vote")]
#[derive(Accounts)]
pub struct CastVoteCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CAST_VOTE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub vote_record: Account<'info, VoteRecord>,
}

#[account]
#[derive(InitSpace)]
pub struct ProgramState {
    pub authority: Pubkey,
    pub total_proposals: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub creator: Pubkey,
    pub proposal_id: u64,
    #[max_len(64)]
    pub title: String,
    #[max_len(256)]
    pub description: String,
    pub num_options: u8,
    #[max_len(8, 32)]
    pub option_labels: [String; 8],
    pub voting_ends_at: i64,
    pub finalized: bool,
    pub results: [u64; 8],
    pub total_votes: u64,
}

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal_id: u64,
    pub voted: bool,
}

#[event]
pub struct VoteEvent { pub voter: Pubkey }

#[error_code]
pub enum ErrorCode {
    #[msg("Computation aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}
