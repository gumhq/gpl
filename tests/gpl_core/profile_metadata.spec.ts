import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Profile Metadata", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;
  let profileMetadataPDA: anchor.web3.PublicKey;

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

  it("should create a profile metadata", async () => {
    const randomHash = randombytes(32);
    const metadataUri = "https://example.com";
    const profileMetadata = program.methods
      .createProfileMetadata(metadataUri)
      .accounts({ user: userPDA, profile: profilePDA });
    const profileMetadataPubKeys = await profileMetadata.pubkeys();
    profileMetadataPDA =
      profileMetadataPubKeys.profileMetadata as anchor.web3.PublicKey;
    await profileMetadata.rpc();
    const profileMetadataAccount = await program.account.profileMetadata.fetch(
      profileMetadataPDA
    );
    expect(profileMetadataAccount.metadataUri).is.equal(metadataUri);
    expect(profileMetadataAccount.profile.toString()).is.equal(
      profilePDA.toString()
    );
  });

  it("should update a profile metadata", async () => {
    const metadataUri = "https://example.com/updated";
    const profileMetadata = program.methods
      .updateProfileMetadata(metadataUri)
      .accounts({
        user: userPDA,
        profile: profilePDA,
        profileMetadata: profileMetadataPDA,
      });
    await profileMetadata.rpc();
    const profileMetadataAccount = await program.account.profileMetadata.fetch(
      profileMetadataPDA
    );
    expect(profileMetadataAccount.metadataUri).is.equal(metadataUri);
    expect(profileMetadataAccount.profile.toString()).is.equal(
      profilePDA.toString()
    );
  });

  it("should delete a profile metadata", async () => {
    const profileMetadata = program.methods.deleteProfileMetadata().accounts({
      user: userPDA,
      profile: profilePDA,
      profileMetadata: profileMetadataPDA,
    });
    await profileMetadata.rpc();
    try {
      await program.account.profileMetadata.fetch(profileMetadataPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist or has no data ${profileMetadataPDA.toString()}`
      );
    }
  });
});
