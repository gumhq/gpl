import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";
import { createGumDomain, createGumTld } from "../utils";
import { new_session } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

const provider = anchor.getProvider();

describe("Reaction", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;
  let postPDA: anchor.web3.PublicKey;
  let reactionPDA: anchor.web3.PublicKey;

  before(async () => {
    // Create a user
    const randomHash = randombytes(32);
    const gumTld = await createGumTld();
    const screenName = await createGumDomain(gumTld, "foobarasdfas");

    // Create a profile
    const profileMetdataUri = "https://example.com";
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
  });

  it("should create a reaction", async () => {
    const reaction = program.methods.createReaction("Haha").accounts({
      toPost: postPDA,
      fromProfile: profilePDA,
      sessionToken: null,
    });
    const reactionPubKeys = await reaction.pubkeys();
    reactionPDA = reactionPubKeys.reaction as anchor.web3.PublicKey;
    await reaction.rpc();

    const reactionAccount = await program.account.reaction.fetch(reactionPDA);
    expect(reactionAccount.toPost.toBase58()).to.equal(postPDA.toBase58());
    expect(reactionAccount.fromProfile.toBase58()).to.equal(
      profilePDA.toBase58()
    );
    expect(reactionAccount.reactionType.toString()).to.equal(
      { haha: {} }.toString()
    );
  });

  it("should delete a reaction", async () => {
    const reaction = program.methods.deleteReaction().accounts({
      toPost: postPDA,
      fromProfile: profilePDA,
      reaction: reactionPDA,
      sessionToken: null,
      refundReceiver: provider.wallet.publicKey,
    });
    await reaction.rpc();

    try {
      await program.account.reaction.fetch(reactionPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist or has no data ${reactionPDA.toString()}`
      );
    }
  });

  describe("Reaction with session token", async () => {
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

    it("should create a reaction", async () => {
      const reaction = program.methods.createReaction("Haha").accounts({
        toPost: postPDA,
        fromProfile: profilePDA,
        sessionToken: sessionToken,
        authority: sessionKeypair.publicKey,
      });
      const reactionPubKeys = await reaction.pubkeys();
      reactionPDA = reactionPubKeys.reaction as anchor.web3.PublicKey;
      await reaction.signers([sessionKeypair]).rpc();
      const reactionAccount = await program.account.reaction.fetch(reactionPDA);
      expect(reactionAccount.toPost.toBase58()).to.equal(postPDA.toBase58());
      expect(reactionAccount.fromProfile.toBase58()).to.equal(
        profilePDA.toBase58()
      );
      expect(reactionAccount.reactionType.toString()).to.equal(
        { haha: {} }.toString()
      );
    });

    it("should delete a reaction", async () => {
      const reaction = program.methods.deleteReaction().accounts({
        toPost: postPDA,
        fromProfile: profilePDA,
        reaction: reactionPDA,
        sessionToken: sessionToken,
        authority: sessionKeypair.publicKey,
        // @ts-ignore
        refundReceiver: provider.wallet.publicKey,
      });

      await reaction.signers([sessionKeypair]).rpc();

      try {
        await program.account.reaction.fetch(reactionPDA);
      } catch (error: any) {
        expect(error).to.be.an("error");
        expect(error.toString()).to.contain(
          `Account does not exist or has no data ${reactionPDA.toString()}`
        );
      }
    });
  });
});
