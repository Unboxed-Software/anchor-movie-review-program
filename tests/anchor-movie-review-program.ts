import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { expect } from "chai"
import { getAssociatedTokenAddress, getAccount } from "@solana/spl-token"
import { AnchorMovieReviewProgram } from "../target/types/anchor_movie_review_program"

describe("anchor-movie-review-program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace
    .AnchorMovieReviewProgram as Program<AnchorMovieReviewProgram>

  const movie = {
    title: "Just a test movie",
    description: "Wow what a good movie it was real great",
    rating: 5,
  }

  const [movie_pda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(movie.title), provider.wallet.publicKey.toBuffer()],
    program.programId
  )

  const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint")],
    program.programId
  )

  const [commentCounterPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("counter"), movie_pda.toBuffer()],
    program.programId
  )

  it("Initializes the reward token", async () => {
    // By using "accountsPartial" we nedd to specify all the accounts
    const tx = await program.methods
      .initializeTokenMint()
      .accountsPartial({
        mint: mint,
      })
      .rpc()
  })

  it("Movie review is added", async () => {
    // Add your test here.
    const tokenAccount = await getAssociatedTokenAddress(
      mint,
      provider.wallet.publicKey
    )

    // By using "accounts" we only need to specify the non-resolvable accounts. The PDAs 'movieReview' and 'movieCommentCounter' as well as the 'mint' are not needed, as they are automatically resolved.
    const tx = await program.methods
      .addMovieReview(movie.title, movie.description, movie.rating)
      .accounts({
        tokenAccount: tokenAccount,
      })
      .rpc()

    const account = await program.account.movieAccountState.fetch(movie_pda)
    expect(account.title).to.equal(movie.title);
    expect(account.rating).to.equal(movie.rating);
    expect(account.description).to.equal(movie.description);
    expect(account.reviewer.toBase58()).to.equal(provider.wallet.publicKey.toBase58())

    const userAta = await getAccount(provider.connection, tokenAccount)
    expect(Number(userAta.amount)).to.equal((10 * 10) ^ 6)
  })

  it("Movie review is updated", async () => {
    const newDescription = "Wow this is new"
    const newRating = 4

    const tx = await program.methods
      .updateMovieReview(movie.title, newDescription, newRating)
      .accountsPartial({
        movieReview: movie_pda,
      })
      .rpc()

    const account = await program.account.movieAccountState.fetch(movie_pda)
    expect(account.title).to.equal(movie.title);
    expect(account.rating).to.equal(newRating);
    expect(account.description).to.equal(newDescription);
    expect(account.reviewer.toBase58()).to.equal(provider.wallet.publicKey.toBase58())
  })

  it("Adds a comment to a movie review", async () => {
    const tokenAccount = await getAssociatedTokenAddress(
      mint,
      provider.wallet.publicKey
    )

    let commentCounter = await program.account.movieCommentCounter.fetch(
      commentCounterPda
    )

    let [commentPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        movie_pda.toBuffer(),
        commentCounter.counter.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )

    const testComment = "Just a test comment";
    const tx = await program.methods
      .addComment(testComment)
      .accountsPartial({
        movieReview: movie_pda,
        mint: mint,
        tokenAccount: tokenAccount,
        movieCommentCounter: commentCounterPda,
        movieComment: commentPda,
      })
      .rpc()

    const commentAccount = await program.account.movieComment.fetch(commentPda);
    expect(commentAccount.comment).to.equal(testComment);
    expect(commentAccount.commenter.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
    expect(commentAccount.count.toNumber()).to.equal(1);

    const commentCounterAccount = await program.account.movieCommentCounter.fetch(commentCounterPda);
    expect(commentCounterAccount.counter.toNumber()).to.equal(1);
  })

  it("Deletes a movie review", async () => {
    const tx = await program.methods
      .deleteMovieReview(movie.title)
      .accountsPartial({ movieReview: movie_pda })
      .rpc()
  })
})
