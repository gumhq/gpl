import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Profile", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;

  before(async () => {
    // Create a user
    const randomHash = randombytes(32);
    const tx = program.methods.createUser(randomHash);
    const pubKeys = await tx.pubkeys();
    userPDA = pubKeys.user as anchor.web3.PublicKey;
    await tx.rpc();
  });

  it("should create a profile", async () => {
    const tx = program.methods
      .createProfile("Personal")
      .accounts({ user: userPDA });
    const pubKeys = await tx.pubkeys();
    profilePDA = pubKeys.profile as anchor.web3.PublicKey;
    await tx.rpc();
    const profileAccount = await program.account.profile.fetch(profilePDA);
    expect(profileAccount.user.toString()).is.equal(userPDA.toString());
    expect(profileAccount.namespace.toString()).is.equal(
      { personal: {} }.toString()
    );
  });

  it("should delete a profile", async () => {
    const tx = program.methods
      .deleteProfile()
      .accounts({ user: userPDA, profile: profilePDA });
    await tx.rpc();
    try {
      await program.account.profile.fetch(profilePDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist ${profilePDA.toString()}`
      );
    }
  });
});

