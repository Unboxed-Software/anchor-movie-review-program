use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, MintTo, Mint, TokenAccount, Token};
use anchor_spl::associated_token::AssociatedToken;

mod constants;

use constants::*;

declare_id!("ET1Gctnke2Gwusu336M2kfaRsfMY9ddg6RHWD3GzJDyN");

#[program]
pub mod anchor_movie_review_program {
    use super::*;

    pub fn add_movie_review(ctx: Context<AddMovieReview>, title: String, description: String, rating: u8) -> Result<()> {

        // We require that the rating is between 1 and 5
        require!(rating >= MIN_RATING && rating <= MAX_RATING, MovieReviewError::InvalidRating);

        // We require that the title is not longer than 20 characters
        require!(title.len() <= MAX_TITLE_LENGTH, MovieReviewError::TitleTooLong);

        // We require that the description is not longer than 50 characters
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, MovieReviewError::DescriptionTooLong);

        msg!("Movie review account created");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);
        
        let movie_review = &mut ctx.accounts.movie_review;
        movie_review.reviewer = ctx.accounts.initializer.key();
        movie_review.title = title;
        movie_review.description = description;
        movie_review.rating = rating;

        msg!("Movie Comment Counter Account Created");
        let movie_comment_counter = &mut ctx.accounts.movie_comment_counter;
        movie_comment_counter.counter = 0;
        msg!("Counter: {}", movie_comment_counter.counter);

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo { 
                    authority: ctx.accounts.mint.to_account_info(), 
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info()
                },
                &[&[
                    "mint".as_bytes().as_ref(),
                    &[ctx.bumps.mint]
                ]]
            ),
            10*10^6
        )?;

        msg!("Minted tokens");
        
        Ok(())
    }

    pub fn add_comment(ctx: Context<AddComment>, comment: String) -> Result<()> {

        // We require that the comment is not longer than 60 characters
        require!(comment.len() <= MAX_COMMENT_LENGTH, MovieReviewError::CommentTooLong);

        msg!("Comment Account Created");
        msg!("Comment: {}", comment);

        let movie_comment = &mut ctx.accounts.movie_comment;
        let movie_comment_counter = &mut ctx.accounts.movie_comment_counter;

        movie_comment.review = ctx.accounts.movie_review.key();
        movie_comment.commenter = ctx.accounts.initializer.key();
        movie_comment.comment = comment;
        movie_comment.count = movie_comment_counter.counter;

        movie_comment_counter.counter += 1;

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint.to_account_info(),
                },
                &[&[
                    "mint".as_bytes().as_ref(),
                    &[ctx.bumps.mint]
                ]]
            ), 
            5*10^6
        )?;
        msg!("Minted Tokens");

        Ok(())
    }

    pub fn update_movie_review(ctx: Context<UpdateMovieReview>, title: String, description: String, rating: u8) -> Result<()> {
        
        // We require that the rating is between 1 and 5
        require!(rating >= MIN_RATING && rating <= MAX_RATING, MovieReviewError::InvalidRating);

        // We require that the description is not longer than 50 characters
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, MovieReviewError::DescriptionTooLong);

        msg!("Movie review account space reallocated");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);
        
        let movie_review = &mut ctx.accounts.movie_review;
        movie_review.description = description;
        movie_review.rating = rating;
        
        Ok(())
    }

    pub fn delete_movie_review(_ctx: Context<DeleteMovieReview>, title: String) -> Result<()> {
        msg!("Movie review for {} deleted", title);
        Ok(())
    }

    pub fn initialize_token_mint(_ctx: Context<InitializeMint>) -> Result<()> {
        msg!("Token mint initialized");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct AddMovieReview<'info> {
    #[account(
        init, 
        seeds=[title.as_bytes(), initializer.key().as_ref()], 
        bump, 
        payer = initializer, 
        space = MovieAccountState::INIT_SPACE + title.len() + description.len() // We add the length of the title and description to the init space
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(
        init,
        seeds = ["counter".as_bytes().as_ref(), movie_review.key().as_ref()],
        bump,
        payer = initializer,
        space = ANCHOR_DISCRIMINATOR + MovieCommentCounter::INIT_SPACE // We add the anchor discriminator to the init space
    )]
    pub movie_comment_counter: Account<'info, MovieCommentCounter>,
    #[account(
        seeds=["mint".as_bytes().as_ref()],
        bump,
        mut
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint,
        associated_token::authority = initializer
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(comment: String)]
pub struct AddComment<'info> {
    #[account(
        init,
        seeds = [movie_review.key().as_ref(), &movie_comment_counter.counter.to_le_bytes()],
        bump,
        payer = initializer,
        space = MovieComment::INIT_SPACE + comment.len() // We add the length of the comment to the init space
    )]
    pub movie_comment: Account<'info, MovieComment>,
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(
        mut,
        seeds = ["counter".as_bytes().as_ref(), movie_review.key().as_ref()],
        bump,
    )]
    pub movie_comment_counter: Account<'info, MovieCommentCounter>,
    #[account(
        mut,
        seeds = ["mint".as_bytes().as_ref()],
        bump
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint,
        associated_token::authority = initializer
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct UpdateMovieReview<'info> {
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
        realloc = MovieAccountState::INIT_SPACE + title.len() + description.len(), // We add the length of the title and description to the init space
        realloc::payer = initializer,
        realloc::zero = true
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteMovieReview<'info> {
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
        close=initializer
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        seeds = ["mint".as_bytes().as_ref()],
        bump,
        payer = user,
        mint::decimals = 6,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>
}

/*
    For the MovieAccountState account, since it is dynamic, we implement the Space trait to calculate the space required for the account.
    We add the STRING_LENGTH_PREFIX twice to the space to account for the title and description string prefix.
    We need to add the length of the title and description to the space upon initialization.
 */
#[account]
pub struct MovieAccountState {
    pub reviewer: Pubkey,
    pub rating: u8,
    pub title: String,
    pub description: String,
}

impl Space for MovieAccountState {
    const INIT_SPACE: usize = ANCHOR_DISCRIMINATOR + PUBKEY_SIZE + U8_SIZE + STRING_LENGTH_PREFIX + STRING_LENGTH_PREFIX;
}


/*
    For the MovieCommentCounter account, since it is not dynamic, we use the InitSpace derive macro to calculate the space required for the account.
    We need to add the anchor discriminator to the space upon initialization.
 */
#[account]
#[derive(InitSpace)]
pub struct MovieCommentCounter {
    pub counter: u64,
}

/*
    For the MovieComment account, since it is dynamic, we implement the Space trait to calculate the space required for the account.
    We add the STRING_LENGTH_PREFIX to the space to account for the comment string prefix.
    We need to add the length of the comment to the space upon initialization.
 */
#[account]
pub struct MovieComment {
    pub review: Pubkey,
    pub commenter: Pubkey,
    pub comment: String,
    pub count: u64,
}

impl Space for MovieComment {
    const INIT_SPACE: usize = ANCHOR_DISCRIMINATOR + PUBKEY_SIZE + PUBKEY_SIZE + STRING_LENGTH_PREFIX + U64_SIZE;
}

#[error_code]
enum MovieReviewError {
    #[msg("Rating must be between 1 and 5")]
    InvalidRating,
    #[msg("Movie Title too long")]
    TitleTooLong,
    #[msg("Movie Description too long")]
    DescriptionTooLong,
    #[msg("Movie Comment too long")]
    CommentTooLong,
}