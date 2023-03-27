import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";
import { new_session } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

const provider = anchor.getProvider();

describe("Post", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;
  let postPDA: anchor.web3.PublicKey;

  before(async () => {
    // Create a user
    const randomHash = randombytes(32);
    const userTx = program.methods.createUser(randomHash);
    const userPubKeys = await userTx.pubkeys();
    userPDA = userPubKeys.user as anchor.web3.PublicKey;
    await userTx.rpc();

    // Create a profile
    const profileTx = program.methods
      .createProfile("Personal")
      .accounts({ user: userPDA });
    const profilePubKeys = await profileTx.pubkeys();
    profilePDA = profilePubKeys.profile as anchor.web3.PublicKey;
    await profileTx.rpc();
  });

  it("should create a post", async () => {
    const randomHash = randombytes(32);
    const metadataUri = "This is a test post";
    const post = program.methods
      .createPost(metadataUri, randomHash)
      .accounts({ user: userPDA, profile: profilePDA, sessionToken: null });
    const postPubKeys = await post.pubkeys();
    postPDA = postPubKeys.post as anchor.web3.PublicKey;
    await post.rpc();
    const postAccount = await program.account.post.fetch(postPDA);
    expect(postAccount.metadataUri).is.equal(metadataUri);
    expect(postAccount.profile.toString()).is.equal(profilePDA.toString());
  });

  it("should update a post", async () => {
    const metadataUri = "This is an updated test post";
    const post = program.methods.updatePost(metadataUri).accounts({
      user: userPDA,
      profile: profilePDA,
      post: postPDA,
      sessionToken: null,
    });
    await post.rpc();
    const postAccount = await program.account.post.fetch(postPDA);
    expect(postAccount.metadataUri).is.equal(metadataUri);
    expect(postAccount.profile.toString()).is.equal(profilePDA.toString());
  });

  it("should delete a post", async () => {
    const post = program.methods.deletePost().accounts({
      user: userPDA,
      profile: profilePDA,
      post: postPDA,
      sessionToken: null,
      refundReceiver: provider.wallet.publicKey,
    });
    await post.rpc();
    try {
      await program.account.post.fetch(postPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist or has no data ${postPDA.toString()}`
      );
    }
  });

  describe("Post with session token", async () => {
    let sessionToken: anchor.web3.PublicKey;
    let sessionKeypair: anchor.web3.Keypair;

    before(async () => {
      // @ts-ignore
      const { sessionPDA, sessionSigner } = await new_session(
        provider.wallet.publicKey,
        program.programId
      );
      sessionToken = sessionPDA;
      sessionKeypair = sessionSigner;
    });

    it("should create a post", async () => {
      const randomHash = randombytes(32);
      const metadataUri = "This is a test post";
      const post = program.methods
        .createPost(metadataUri, randomHash)
        .accounts({
          user: userPDA,
          profile: profilePDA,
          sessionToken: sessionToken,
          authority: sessionKeypair.publicKey,
        });
      const postPubKeys = await post.pubkeys();
      postPDA = postPubKeys.post as anchor.web3.PublicKey;
      await post.signers([sessionKeypair]).rpc();
      const postAccount = await program.account.post.fetch(postPDA);
      expect(postAccount.metadataUri).is.equal(metadataUri);
      expect(postAccount.profile.toString()).is.equal(profilePDA.toString());
    });

    it("should update a post", async () => {
      const metadataUri = "This is an updated test post";
      const post = program.methods.updatePost(metadataUri).accounts({
        user: userPDA,
        profile: profilePDA,
        post: postPDA,
        sessionToken: sessionToken,
        authority: sessionKeypair.publicKey,
      });
      await post.signers([sessionKeypair]).rpc();
      const postAccount = await program.account.post.fetch(postPDA);
      expect(postAccount.metadataUri).is.equal(metadataUri);
      expect(postAccount.profile.toString()).is.equal(profilePDA.toString());
    });

    it("should delete a post", async () => {
      const post = program.methods.deletePost().accounts({
        user: userPDA,
        profile: profilePDA,
        post: postPDA,
        sessionToken: sessionToken,
        authority: sessionKeypair.publicKey,
        refundReceiver: provider.wallet.publicKey,
      });
      await post.signers([sessionKeypair]).rpc();
      try {
        await program.account.post.fetch(postPDA);
      } catch (error: any) {
        expect(error).to.be.an("error");
        expect(error.toString()).to.contain(
          `Account does not exist or has no data ${postPDA.toString()}`
        );
      }
    });
  });
});
