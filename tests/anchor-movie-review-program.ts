import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { AnchorMovieReviewProgram } from "../target/types/anchor_movie_review_program";

describe("Anchor Movie Review Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorMovieReviewProgram as Program<AnchorMovieReviewProgram>;

  const movie = {
    title: "Just a test movie",
    description: "Wow what a good movie it was real great",
    rating: 5,
  };

  const [moviePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(movie.title), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  it("adds a movie review", async () => {
    try {
      await program.methods
        .addMovieReview(movie.title, movie.description, movie.rating)
        .rpc();

      const account = await program.account.movieAccountState.fetch(moviePda);
      expect(account.title).to.equal(movie.title);
      expect(account.rating).to.equal(movie.rating);
      expect(account.description).to.equal(movie.description);
      expect(account.reviewer.toString()).to.equal(provider.wallet.publicKey.toString());
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
      expect(account.reviewer.toString()).to.equal(provider.wallet.publicKey.toString());
    } catch (error) {
      console.error("Error updating movie review:", error);
      throw error;
    }
  });

  it("deletes a movie review", async () => {
    try {
      await program.methods.deleteMovieReview(movie.title).rpc();
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