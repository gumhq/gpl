import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCore } from "../../target/types/gpl_core";
import { airdrop, createGumDomain, createGumTld } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Profile", async () => {
  let profilePDA: anchor.web3.PublicKey;
  let gumTld: anchor.web3.PublicKey;
  let feePayer: anchor.web3.Keypair;

  before(async () => {
    // Create gum tld
    gumTld = await createGumTld();
    feePayer = anchor.web3.Keypair.generate();
    await airdrop(feePayer.publicKey);
  });

  it("should create a profile", async () => {
    const profileMetdataUri = "https://example.com";
    const screenName = await createGumDomain(gumTld, "foobar123123");
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
        `Account does not exist or has no data ${profilePDA.toString()}`
      );
    }
  });

  it("should create a profile when a seperate fee payer is specified", async () => {
    const randomHash = randombytes(32);
    const profileMetdataUri = "https://example.com";
    const screenName = await createGumDomain(gumTld, "dfgdsfgsdfgsrd");
    const tx = program.methods
      .createProfile(randomHash, profileMetdataUri)
      .accounts({ payer: feePayer.publicKey, screenName });
    const pubKeys = await tx.pubkeys();
    profilePDA = pubKeys.profile as anchor.web3.PublicKey;
    await tx.signers([feePayer]).rpc();
    const profileAccount = await program.account.profile.fetch(profilePDA);
    expect(profileAccount.metadataUri).to.equal(profileMetdataUri);
  });
});
