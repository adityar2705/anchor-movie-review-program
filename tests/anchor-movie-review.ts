import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getAssociatedTokenAddress, getAccount } from "@solana/spl-token"
import { MovieReviewProgram } from "../target/types/movie_review_program";

describe("anchor-movie-review", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MovieReviewProgram as Program<MovieReviewProgram>;

  //test movie review
  const movie = {
    title: "Napoleon",
    description: "Great biopic based on the life of Napoleon Bonaparte.",
    rating: 5,
  }

  //deriving the movie PDA for storage
  const [moviePda] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from(movie.title),
    //getting the initializer address
    provider.wallet.publicKey.toBuffer()
  ],program.programId);

  //deriving the PDA for the mint token account
  const [mint] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("mint"),
  ],program.programId);

  it("Initializes the reward token", async () => {
    const tx = await program.methods.initializeTokenMint().rpc()
  })

  it("Movie review is added`", async () => {
     //get the token account of the particular user
     const tokenAccount = await getAssociatedTokenAddress(
      mint,
      provider.wallet.publicKey
    );
    
    const tx = await program.methods
      .addMovieReview(movie.title,movie.description,movie.rating)
      .rpc()
    
    //getting the PDA storage account
    const account = await program.account.movieAccountState.fetch(moviePda);
    console.log("Movie review successfully added.");
    expect(movie.title === account.title)
    expect(movie.rating === account.rating)
    expect(movie.description === account.description)

    //expect the reviewer (initializer) to be set as the provider wallet public key
    expect(account.reviewer === provider.wallet.publicKey)

    //checking the user token balance
    const userTokens = await getAccount(provider.connection,tokenAccount);
    console.log(`User has received minted tokens for movie review.`);
    expect(Number(userTokens.amount)).to.equal((10 * 10) ^ 6)
  })

  it("Movie review is updated`", async () => {
    const newDescription = "Wow this is new"
    const newRating = 4
    
    //calling the update function
    const tx = await program.methods
      .updateMovieReview(movie.title, newDescription, newRating)
      .rpc()
    
    const account = await program.account.movieAccountState.fetch(moviePda)

    //printing the updated movie review data
    console.log("Movie review successfully updated.");
    expect(movie.title === account.title)
    expect(newRating === account.rating)
    expect(newDescription === account.description)
    expect(account.reviewer === provider.wallet.publicKey)
  })

  it("Deletes a movie review", async () => {
    const tx = await program.methods
    .deleteMovieReview(movie.title)
    .rpc()
  })
  
});
