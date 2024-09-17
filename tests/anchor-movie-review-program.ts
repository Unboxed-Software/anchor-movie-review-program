import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getAssociatedTokenAddress, getAccount } from "@solana/spl-token";
import { AnchorMovieReviewProgram } from "../target/types/anchor_movie_review_program";

describe("Anchor Movie Review Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .AnchorMovieReviewProgram as Program<AnchorMovieReviewProgram>;

  const movie = {
    title: "Just a test movie",
    description: "Wow what a good movie it was real great",
    rating: 5,
  };

  const [moviePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(movie.title), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint")],
    program.programId
  );

  let tokenAccount: anchor.web3.PublicKey;

  before(async () => {
    tokenAccount = await getAssociatedTokenAddress(
      mint,
      provider.wallet.publicKey
    );
  });

  it("initializes the reward token", async () => {
    try {
      await program.methods.initializeTokenMint().rpc();
    } catch (error) {
      console.error("Error initializing token mint:", error);
      throw error;
    }
  });

  it("adds a movie review", async () => {
    try {
      await program.methods
        .addMovieReview(movie.title, movie.description, movie.rating)
        .accounts({
          tokenAccount: tokenAccount,
        })
        .rpc();

      const account = await program.account.movieAccountState.fetch(moviePda);
      expect(account.title).to.equal(movie.title);
      expect(account.rating).to.equal(movie.rating);
      expect(account.description).to.equal(movie.description);
      expect(account.reviewer.toString()).to.equal(
        provider.wallet.publicKey.toString()
      );

      const userAta = await getAccount(provider.connection, tokenAccount);
      expect(Number(userAta.amount)).to.equal((10 * 10) ^ 6);
    } catch (error) {
      console.error("Error adding movie review:", error);
      throw error;
    }
  });

  it("updates a movie review", async () => {
    const newDescription = "Wow this is new";
    const newRating = 4;

    try {
      await program.methods
        .updateMovieReview(movie.title, newDescription, newRating)
        .rpc();

      const account = await program.account.movieAccountState.fetch(moviePda);
      expect(account.title).to.equal(movie.title);
      expect(account.rating).to.equal(newRating);
      expect(account.description).to.equal(newDescription);
      expect(account.reviewer.toString()).to.equal(
        provider.wallet.publicKey.toString()
      );
    } catch (error) {
      console.error("Error updating movie review:", error);
      throw error;
    }
  });

  it("deletes a movie review", async () => {
    try {
      await program.methods.deleteMovieReview(movie.title).rpc();

      // Optionally, verify that the account was deleted
      try {
        await program.account.movieAccountState.fetch(moviePda);
        throw new Error("Account should have been deleted");
      } catch (error) {
        expect(error.message).to.include("Account does not exist");
      }
    } catch (error) {
      console.error("Error deleting movie review:", error);
      throw error;
    }
  });
});
