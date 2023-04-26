import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";
import { createGumDomain, createGumTld } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Profile", async () => {
  let profilePDA: anchor.web3.PublicKey;
  let gumTld: anchor.web3.PublicKey;

  before(async () => {
    // Create gum tld
    gumTld = await createGumTld();
  });

  it("should create a profile", async () => {
    const profileMetdataUri = "https://example.com";
    const screenName = await createGumDomain(gumTld, "foobar123123");
    // Create a user
    const randomHash = randombytes(32);

    const tx = program.methods
      .createProfile(randomHash, profileMetdataUri)
      .accounts({ screenName });
    const pubKeys = await tx.pubkeys();
    profilePDA = pubKeys.profile as anchor.web3.PublicKey;
    await tx.rpc();
    const profileAccount = await program.account.profile.fetch(profilePDA);
    expect(profileAccount.metadataUri).to.equal(profileMetdataUri);
  });

  it("should delete a profile", async () => {
    const tx = program.methods
      .deleteProfile()
      .accounts({ profile: profilePDA });
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
