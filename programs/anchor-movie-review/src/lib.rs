use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::Token;
use anchor_spl::token::{Mint,mint_to,MintTo};

declare_id!("EoqoSfLhdgvr9YxPcu17apnZ6vroTXfTFMskA8dvpMV2");

//anchor-spl = ">=0.30.0, <0.30.1"
//test = "ts-mocha -p ./tsconfig.json 'tests/**/*.ts'"
#[program]
pub mod movie_review_program {
    use super::*;

    //function to add the movie review
    pub fn add_movie_review(
        ctx : Context<AddMovieReview>,
        title : String,
        description: String,
        rating : u8
    ) -> Result<()>{
        msg!("Movie review account created.");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);

        //using our custom error
        require!(rating >= 1 && rating <=5 , MovieReviewError::InvalidRating);

        //movie_review is present in the AddMovieReview struct
        let movie_review = &mut ctx.accounts.movie_review;
        movie_review.reviewer = ctx.accounts.initializer.key();
        movie_review.title = title;
        movie_review.rating = rating;
        movie_review.description = description;

        //minting the tokens to our users using the CPI calls
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo{
                    //as stated in tut -> we are using mint PDA account both as the mint account and also the mint authority
                    authority:ctx.accounts.mint.to_account_info(),
                    to:ctx.accounts.token_account.to_account_info(),
                    mint:ctx.accounts.mint.to_account_info()
                },
                &[&[
                    "mint".as_bytes(),
                    &[ctx.bumps.mint]
                ]]
            ),
            10*10^6
        )?;

        msg!("Minted the tokens to the user");
        Ok(())
    }

    //function to update the movie review
    pub fn update_movie_review(
        ctx: Context<UpdateMovieReview>,
        title: String,
        description: String,
        rating: u8,
    ) -> Result<()>{
        //reallocating space when the review is updated
        msg!("Movie review account space reallocated");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);

        //using custom anchor error
        require!(rating >= 1 && rating <= 5, MovieReviewError::InvalidRating);

        let movie_review =  &mut ctx.accounts.movie_review;
        movie_review.rating = rating;
        movie_review.description = description;

        Ok(())
    }

    //function to delete a movie review
    pub fn delete_movie_review(
        _ctx:Context<DeleteMovieReview>,
        title:String
    ) -> Result<()>{
        msg!("Movie review for {} deleted", title);
        Ok(())
    }

    //mint a token every time a new movie review is added
    pub fn initialize_token_mint(_ctx: Context<InitializeMint>)->Result<()>{
        msg!("Token mint initialized");
        Ok(())
    }

}

//movie account state -> represents the structure for our PDA
#[account]
pub struct MovieAccountState{
    pub reviewer :Pubkey,
    pub rating : u8,
    pub title : String,
    pub description : String
}

//overriding INIT_SPACE
impl Space for MovieAccountState{
    //string length twice -> Borsh needs 4 bytes to store the string length
    const INIT_SPACE : usize = 1000;
}

//add movie review accounts struct
#[derive(Accounts)]
#[instruction(title : String, description : String)]
pub struct AddMovieReview<'info>{
    #[account(
        init,
        seeds = [title.as_bytes(), initializer.key.as_ref()],
        bump,
        payer = initializer,
        space = MovieAccountState::INIT_SPACE + title.len() + description.len()
    )]
    pub movie_review : Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer : Signer<'info>,

    //required for initiliazing the PDA using init, payer and space
    pub system_program : Program<'info,System>,
    pub token_program : Program<'info,Token>,

    //getting the mint PDA account
    #[account(
        seeds = ["mint".as_bytes()],
        bump,
        mut
    )]
    //mint account to send the tokens -> when user submits a movie review
    pub mint : Account<'info,Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint,
        associated_token::authority = initializer
    )]
    //creating a token account -> like a bank account for every user and mint token is like the account holding the currency
    pub token_account : Account<'info,TokenAccount>,
    pub associated_token_program : Program<'info,AssociatedToken>,
    pub rent : Sysvar<'info,Rent>,
}

//update movie review accounts struct
#[derive(Accounts)]
#[instruction(title:String, description:String)]
pub struct UpdateMovieReview<'info>{
    #[account(
        mut,
        seeds = [title.as_bytes(), initializer.key.as_ref()],
        bump,
        realloc = MovieAccountState::INIT_SPACE + title.len() + description.len(),
        realloc::payer = initializer,
        realloc::zero = true,
    )]
    pub movie_review: Account<'info, MovieAccountState>,

    //using the same accounts as before
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//delete movie review accounts struct
#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteMovieReview<'info>{
    //we add the seeds and bump for the required validation
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,

        //using the initializer to use the close function -> rent should be refunded to the initializer account
        close=initializer
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>
}

// initialize mint struct
#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        seeds = ["mint".as_bytes()],
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

//creating a new anchor error
#[error_code]
enum MovieReviewError{
    #[msg("Rating should be between 1 and 5")]
    InvalidRating,
}