import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { sendAndConfirmTransaction } from "@solana/web3.js";
import { GplCore } from "../../target/types/gpl_core";
import { new_session, airdrop, createGumTld, createGumDomain } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

const provider = anchor.getProvider();

describe("Comment", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;
  let postPDA: anchor.web3.PublicKey;
  let testUserKeypair: anchor.web3.Keypair;
  let feePayer: anchor.web3.Keypair;
  let testUserPDA: anchor.web3.PublicKey;
  let fromProfilePDA: anchor.web3.PublicKey;

  before(async () => {
    const randomHash = randombytes(32);
    const gumTld = await createGumTld();

    // Create a profile
    const profileMetdataUri = "https://example.com";
    const screenName = await createGumDomain(gumTld, "foobarq3eqw");
    const profileTx = program.methods
      .createProfile(randomHash, profileMetdataUri)
      .accounts({ screenName });
    const profilePubKeys = await profileTx.pubkeys();
    profilePDA = profilePubKeys.profile as anchor.web3.PublicKey;
    await profileTx.rpc();

    // Create a post
    const postRandomHash = randombytes(32);
    const metadataUri = "This is a test post";
    const post = program.methods
      .createPost(metadataUri, postRandomHash)
      .accounts({ profile: profilePDA, sessionToken: null });
    const postPubKeys = await post.pubkeys();
    postPDA = postPubKeys.post as anchor.web3.PublicKey;
    await post.rpc();

    // Create a test user keypair
    testUserKeypair = anchor.web3.Keypair.generate();
    await airdrop(testUserKeypair.publicKey);

    // Create fee payer keypair
    feePayer = anchor.web3.Keypair.generate();
    await airdrop(feePayer.publicKey);

    // Create a test user pda
    const testUserRandomhash = randombytes(32);
    const testScreenName = await createGumDomain(
      gumTld,
      "goobarq3eqw",
      testUserKeypair
    );

    // Create a from profile
    const fromProfileTx = program.methods
      .createProfile(testUserRandomhash, profileMetdataUri)
      .accounts({
        authority: testUserKeypair.publicKey,
        screenName: testScreenName,
      });
    const fromProfilePubkeys = await fromProfileTx.pubkeys();
    fromProfilePDA = fromProfilePubkeys.profile as anchor.web3.PublicKey;
    await fromProfileTx.signers([testUserKeypair]).rpc();
  });

  it("should create a comment", async () => {
    // create a comment from fromProfilePDA to postPDA
    const commentTx = program.methods
      // @ts-ignore
      .createComment("This is a test comment", randombytes(32))
      .accounts({
        replyTo: postPDA,
        profile: fromProfilePDA,
        authority: testUserKeypair.publicKey,
        sessionToken: null,
      });

    const commentPubkeys = await commentTx.pubkeys();
    const commentPDA = commentPubkeys.post as anchor.web3.PublicKey;
    await commentTx.signers([testUserKeypair]).rpc();
    const commentAccount = await program.account.post.fetch(commentPDA);
    expect(commentAccount.profile.toString()).is.equal(
      fromProfilePDA.toString()
    );
  });

  it("should create a comment when a seperate fee payer is specified", async () => {
    const createComment = program.methods
      // @ts-ignore
      .createComment("This is a test comment", randombytes(32))
      .accounts({
        payer: feePayer.publicKey,
        replyTo: postPDA,
        profile: fromProfilePDA,
        user: testUserPDA,
        authority: testUserKeypair.publicKey,
        sessionToken: null,
      });
    const pubKeys = await createComment.pubkeys();
    const commentPDA = pubKeys.post as anchor.web3.PublicKey;
    await createComment.signers([feePayer, testUserKeypair]).rpc();
    const commentAccount = await program.account.post.fetch(commentPDA);
    expect(commentAccount.profile.toString()).is.equal(
      fromProfilePDA.toString()
    );
  });

  describe("Comment with session token", async () => {
    let sessionToken: anchor.web3.PublicKey;
    let sessionKeypair: anchor.web3.Keypair;

    before(async () => {
      // @ts-ignore
      const { sessionPDA, sessionSigner } = await new_session(
        testUserKeypair.publicKey,
        program.programId,
        testUserKeypair
      );
      sessionToken = sessionPDA;
      sessionKeypair = sessionSigner;
    });

    it("should create a comment", async () => {
      // create a comment from fromProfilePDA to postPDA
      const commentTx = program.methods
        // @ts-ignore
        .createComment("This is a test comment", randombytes(32))
        .accounts({
          replyTo: postPDA,
          profile: fromProfilePDA,
          authority: sessionKeypair.publicKey,
          sessionToken: sessionToken,
        });

      const commentPubkeys = await commentTx.pubkeys();
      const commentPDA = commentPubkeys.post as anchor.web3.PublicKey;
      await commentTx.signers([sessionKeypair]).rpc();
      const commentAccount = await program.account.post.fetch(commentPDA);
      expect(commentAccount.profile.toString()).is.equal(
        fromProfilePDA.toString()
      );
    });
  });
});
