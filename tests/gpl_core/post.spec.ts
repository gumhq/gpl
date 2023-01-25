import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

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
    const postMetadata = "text/plain";
    const post = program.methods.createPost(metadataUri, postMetadata, randomHash).accounts({ user: userPDA, profile: profilePDA });
    const postPubKeys = await post.pubkeys();
    postPDA = postPubKeys.post as anchor.web3.PublicKey;
    await post.rpc();
    const postAccount = await program.account.post.fetch(postPDA);
    expect(postAccount.metadataUri).is.equal(metadataUri);
    expect(postAccount.postMetadata).is.equal(postMetadata);
    expect(postAccount.profile.toString()).is.equal(profilePDA.toString());
  });

  it("should update a post", async () => {
    const metadataUri = "This is an updated test post";
    const postMetadata = "text/plain";
    const post = program.methods.updatePost(metadataUri, postMetadata).accounts({ user: userPDA, profile: profilePDA, post: postPDA });
    await post.rpc();
    const postAccount = await program.account.post.fetch(postPDA);
    expect(postAccount.metadataUri).is.equal(metadataUri);
    expect(postAccount.postMetadata).is.equal(postMetadata);
    expect(postAccount.profile.toString()).is.equal(profilePDA.toString());
  });

  it("should delete a post", async () => {
    const post = program.methods
      .deletePost()
      .accounts({ user: userPDA, profile: profilePDA, post: postPDA });
    await post.rpc();
    try {
      await program.account.post.fetch(postPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist ${postPDA.toString()}`
      );
    }
  });
});

