import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";
import { airdrop } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Profile", async () => {
  let userPDA: anchor.web3.PublicKey;
  let profilePDA: anchor.web3.PublicKey;
  let feePayer: anchor.web3.Keypair;

  before(async () => {
    // Create a user
    const randomHash = randombytes(32);
    const tx = program.methods.createUser(randomHash);
    const pubKeys = await tx.pubkeys();
    userPDA = pubKeys.user as anchor.web3.PublicKey;
    await tx.rpc();

    // Create fee payer keypair
    feePayer = anchor.web3.Keypair.generate();
    await airdrop(feePayer.publicKey);
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
        `Account does not exist or has no data ${profilePDA.toString()}`
      );
    }
  });

  it("should create a profile when a seperate fee payer is specified", async () => {
    const tx = program.methods
      .createProfile("Personal")
      .accounts({ payer: feePayer.publicKey, user: userPDA });
    const pubKeys = await tx.pubkeys();
    profilePDA = pubKeys.profile as anchor.web3.PublicKey;
    await tx.signers([feePayer]).rpc();
    const profileAccount = await program.account.profile.fetch(profilePDA);
    expect(profileAccount.user.toString()).is.equal(userPDA.toString());
    expect(profileAccount.namespace.toString()).is.equal(
      { personal: {} }.toString()
    );
  });
});
