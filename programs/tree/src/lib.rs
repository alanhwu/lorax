use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("11111111111111111111111111111111");

// Minimum balance to create a tree
const MINIMUM_LAMPORTS: u64 = 5000; // todo
const SPACE: u64 = 128; // todo: decide depth to determine space

const CREATE_TREE_INSTRUCTION_DISCRIMINATOR: &[u8] = &[165, 83, 136, 142, 89, 202, 47, 220]; //

pub static mut INDEX: u64 = 0;

#[program]
mod tree {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        ctx.accounts.counter.count = 0;
        Ok(())
    }

    // Perform check and call compression program
    pub fn create_tree(ctx: Context<CreateTree>) -> ProgramResult {

        //use the sysvar to check the cost in rent. it's dynamic solana_program::rent
        let lamports_ref = ctx.accounts.tree_creator.lamports.borrow_mut(); // Borrow the RefCell mutably
        let lamports_value = *lamports_ref; // Dereference to get the u64 value
        
        if *lamports_value < MINIMUM_LAMPORTS {
            return Err(anchor_lang::prelude::ProgramError::Custom(TreeInitializationFailure::InsufficientFunds as u32));
        }
        
        
        
        // cpi stuff
        // I need to do an allocation first. That's systemprogram create account.
        // and then somehow call the createTree on the spl account compression program. also need to grab noop program

        // Allocate a tree account
        let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.new_account.key(),
            MINIMUM_LAMPORTS,
            SPACE,
            &ctx.accounts.compression_program.key(),
        );
        invoke(
            &create_account_ix,
            &[
                &ctx.accounts.new_account,
                &ctx.accounts.payer,
                &ctx.accounts.system_program,
            ],
        )?;

        // Serialize the instruction data (arguments)
        let max_depth: u32 = 5; //should be big!
        let max_buffer_size: u32 = 256; // need to choose this strategically
        let public: bool = false;
        let mut instruction_data = Vec::from(CREATE_TREE_INSTRUCTION_DISCRIMINATOR);
        instruction_data.extend_from_slice(&max_depth.to_le_bytes()); //solana uses little endian!
        instruction_data.extend_from_slice(&max_buffer_size.to_le_bytes());
        instruction_data.push(public as u8);

        let account_metas = vec![
            AccountMeta::new(*ctx.accounts.tree_authority.to_account_info().key, false),
            AccountMeta::new(*ctx.accounts.merkle_tree.to_account_info().key, true),
            AccountMeta::new(*ctx.accounts.payer.to_account_info().key, true),
            AccountMeta::new(*ctx.accounts.tree_creator.to_account_info().key, false),
            AccountMeta::new(*ctx.accounts.log_wrapper.to_account_info().key, false),
            AccountMeta::new(
                *ctx.accounts.compression_program.to_account_info().key,
                false,
            ),
        ];

        let create_tree_instruction = Instruction {
            program_id: ctx.accounts.compression_program.key(),
            accounts: account_metas,
            data: instruction_data,
        };

        // Perform the CPI
        invoke(
            &create_tree_instruction,
            &[
                ctx.accounts.tree_authority.to_account_info(),
                ctx.accounts.merkle_tree.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.tree_creator.to_account_info(),
                ctx.accounts.log_wrapper.to_account_info(),
                ctx.accounts.compression_program.to_account_info(),
            ],
        )?; // ? will unwrap T from Ok(T) if invoke returns that. error propogation

        Ok(())
    }

    // pub fn mint_to_leaf(ctx: Context<MintToLeaf>, data: u64) -> ProgramResult {
    //     // take index, mint to leaf
    //     Ok(())
    // }
}
#[error_code]
pub enum TreeInitializationFailure {
    #[msg("Insufficient funds to initialize a new tree")]
    InsufficientFunds,
}

#[account]
pub struct Counter {
    pub count: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PerformCpi<'info> {
    // Add CPI Accounts
    #[account(mut)]
    pub my_program_account: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct CreateTree<'info> {
    #[account(init, payer = payer, space = SPACE)]
    pub new_account: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: AccountInfo<'info>,
    pub tree_authority: AccountInfo<'info>,
    #[account(mut)]
    pub merkle_tree: AccountInfo<'info>,
    pub tree_creator: AccountInfo<'info>,
    pub log_wrapper: AccountInfo<'info>,
    pub compression_program: AccountInfo<'info>,
}
