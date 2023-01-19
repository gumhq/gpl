import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Reaction", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;
  let postPDA: anchor.web3.PublicKey;
  let reactionPDA: anchor.web3.PublicKey;

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

    // Create a post
    const postRandomHash = randombytes(32);
    const metadataUri = "This is a test post";
    const post = program.methods
      .createPost(metadataUri, postRandomHash)
      .accounts({ user: userPDA, profile: profilePDA });
    const postPubKeys = await post.pubkeys();
    postPDA = postPubKeys.post as anchor.web3.PublicKey;
    await post.rpc();
  });

  it("should create a reaction", async () => {
    const reaction = program.methods
      .createReaction("Haha")
      .accounts({ toPost: postPDA, fromProfile: profilePDA, user: userPDA });
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
      user: userPDA,
      reaction: reactionPDA,
    });
    await reaction.rpc();

    try {
      await program.account.reaction.fetch(reactionPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist ${reactionPDA.toString()}`
      );
    }
  });
});

