import * as anchor from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import randombytes from "randombytes";
import { airdrop, new_session } from "../utils";
import { expect } from "chai";
import { sendAndConfirmTransaction } from "@solana/web3.js";
import { GplCore } from "../../target/types/gpl_core";
import { createGumDomain, createGumTld } from "../utils";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;
const provider = anchor.getProvider();

anchor.setProvider(anchor.AnchorProvider.env());

describe("Connection", async () => {
  let rpcConnection: anchor.web3.Connection;
  let testUser: anchor.web3.Keypair;
  let feePayer: anchor.web3.Keypair;
  let testUserWallet: NodeWallet;
  let profilePDA: anchor.web3.PublicKey;
  let testProfilePDA: anchor.web3.PublicKey;
  let connectionPDA: anchor.web3.PublicKey;

  before(async () => {
    rpcConnection = new anchor.web3.Connection(
      "http://localhost:8899",
      "confirmed"
    );
    // Create a user
    const randomHash = randombytes(32);

    const gumTld = await createGumTld();
    const screenName = await createGumDomain(gumTld, "foobar");

    // Create a profile
    const profileMetdataUri = "https://example.com";
    const profileTx = program.methods
      .createProfile(randomHash, profileMetdataUri)
      .accounts({ screenName });
    const profilePubKeys = await profileTx.pubkeys();
    profilePDA = profilePubKeys.profile as anchor.web3.PublicKey;
    await profileTx.rpc();

    // Create a testUser
    testUser = anchor.web3.Keypair.generate();
    testUserWallet = new NodeWallet(testUser);
    await airdrop(testUser.publicKey);

    // Create a feePayer
    feePayer = anchor.web3.Keypair.generate();
    await airdrop(feePayer.publicKey);

    // Create a testProfile
    const testProfileMetdataUri = "https://example.com";
    const testRandomHash = randombytes(32);
    const testScreenName = await createGumDomain(gumTld, "test", testUser);
    const testProfile = program.methods
      .createProfile(testRandomHash, testProfileMetdataUri)
      .accounts({
        payer: testUser.publicKey,
        authority: testUser.publicKey,
        screenName: testScreenName,
      })
      .signers([testUser]);
    const testProfilePubKeys = await testProfile.pubkeys();
    testProfilePDA = testProfilePubKeys.profile as anchor.web3.PublicKey;
    await testProfile.rpc();
  });

  it("should create a connection", async () => {
    const connection = program.methods.createConnection().accounts({
      fromProfile: profilePDA,
      toProfile: testProfilePDA,
      sessionToken: null,
    });
    const pubKeys = await connection.pubkeys();
    connectionPDA = pubKeys.connection as anchor.web3.PublicKey;
    await connection.rpc();

    const connectionAccount = await program.account.connection.fetch(
      connectionPDA
    );
    expect(connectionAccount.fromProfile.toBase58()).to.equal(
      profilePDA.toBase58()
    );
    expect(connectionAccount.toProfile.toBase58()).to.equal(
      testProfilePDA.toBase58()
    );
  });

  it("should delete a connection", async () => {
    const connection = program.methods.deleteConnection().accounts({
      fromProfile: profilePDA,
      toProfile: testProfilePDA,
      connection: connectionPDA,
      sessionToken: null,
      // @ts-ignore
      refundReceiver: provider.wallet.publicKey,
    });
    await connection.rpc();

    try {
      await program.account.connection.fetch(connectionPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist or has no data ${connectionPDA.toString()}`
      );
    }
  });

  it("should create a connection when a seperate fee payer is specified", async () => {
    const createConnection = program.methods.createConnection().accounts({
      payer: feePayer.publicKey,
      fromProfile: profilePDA,
      toProfile: testProfilePDA,
      sessionToken: null,
    });
    const pubKeys = await createConnection.pubkeys();
    connectionPDA = pubKeys.connection as anchor.web3.PublicKey;
    await createConnection.signers([feePayer]).rpc();

    const connectionAccount = await program.account.connection.fetch(
      connectionPDA
    );
    expect(connectionAccount.fromProfile.toBase58()).to.equal(
      profilePDA.toBase58()
    );
    expect(connectionAccount.toProfile.toBase58()).to.equal(
      testProfilePDA.toBase58()
    );

    // Cleanup for next tests
    await program.methods
      .deleteConnection()
      .accounts({
        fromProfile: profilePDA,
        toProfile: testProfilePDA,
        connection: connectionPDA,
        sessionToken: null,
        // @ts-ignore
        refundReceiver: provider.wallet.publicKey,
      })
      .rpc();
  });

  describe("Connection with session token", async () => {
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

    it("should create a connection", async () => {
      const connection = program.methods.createConnection().accounts({
        fromProfile: profilePDA,
        toProfile: testProfilePDA,
        sessionToken: sessionToken,
        authority: sessionKeypair.publicKey,
      });
      const pubKeys = await connection.pubkeys();
      connectionPDA = pubKeys.connection as anchor.web3.PublicKey;
      await connection.signers([sessionKeypair]).rpc();
      const connectionAccount = await program.account.connection.fetch(
        connectionPDA
      );
      expect(connectionAccount.fromProfile.toBase58()).to.equal(
        profilePDA.toBase58()
      );
      expect(connectionAccount.toProfile.toBase58()).to.equal(
        testProfilePDA.toBase58()
      );
    });

    it("should delete a connection", async () => {
      const connection = program.methods.deleteConnection().accounts({
        fromProfile: profilePDA,
        toProfile: testProfilePDA,
        connection: connectionPDA,
        sessionToken: sessionToken,
        authority: sessionKeypair.publicKey,
        // @ts-ignore
        refundReceiver: provider.wallet.publicKey,
      });
      await connection.signers([sessionKeypair]).rpc();

      try {
        await program.account.connection.fetch(connectionPDA);
      } catch (error: any) {
        expect(error).to.be.an("error");
        expect(error.toString()).to.contain(
          `Account does not exist or has no data ${connectionPDA.toString()}`
        );
      }
    });
  });
});
