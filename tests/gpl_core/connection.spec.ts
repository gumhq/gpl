import * as anchor from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import randombytes from "randombytes";
import { airdrop } from "../utils";
import { expect } from "chai";
import { sendAndConfirmTransaction } from "@solana/web3.js";
import { GplCore } from "../../target/types/gpl_core";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());

describe("Connection", async () => {
  let rpcConnection: anchor.web3.Connection;
  let testUser: anchor.web3.Keypair;
  let testUserWallet: NodeWallet;
  let userPDA: anchor.web3.PublicKey;
  let testUserPDA: anchor.web3.PublicKey;
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
    const userTx = program.methods.createUser(randomHash);
    const userPubKeys = await userTx.pubkeys();
    userPDA = userPubKeys.user as anchor.web3.PublicKey;
    await userTx.rpc();

    // Create a profile
    const profileMetdataUri = "https://example.com";
    const screenName = anchor.web3.PublicKey.default;
    const profileTx = program.methods
      .createProfile("Personal", profileMetdataUri)
      .accounts({ user: userPDA, screenName });
    const profilePubKeys = await profileTx.pubkeys();
    profilePDA = profilePubKeys.profile as anchor.web3.PublicKey;
    await profileTx.rpc();

    // Create a testUser
    testUser = anchor.web3.Keypair.generate();
    testUserWallet = new NodeWallet(testUser);
    await airdrop(testUser.publicKey);

    const randomTestHash = randombytes(32);
    const createTestUser = program.methods
      .createUser(randomTestHash)
      .accounts({ authority: testUser.publicKey });
    const testUserPubKeys = await createTestUser.pubkeys();
    testUserPDA = testUserPubKeys.user as anchor.web3.PublicKey;
    const testUserTx = await createTestUser.transaction();
    testUserTx.recentBlockhash = (
      await rpcConnection.getLatestBlockhash()
    ).blockhash;
    testUserTx.feePayer = testUser.publicKey;
    const signedTestUserTransaction = await testUserWallet.signTransaction(
      testUserTx
    );
    await sendAndConfirmTransaction(rpcConnection, signedTestUserTransaction, [
      testUser,
    ]);

    // Create a testProfile
    const testProfileMetdataUri = "https://example.com";
    const testScreenName = anchor.web3.PublicKey.default;
    const testProfile = program.methods
      .createProfile("Personal", testProfileMetdataUri)
      .accounts({
        user: testUserPDA,
        authority: testUser.publicKey,
        screenName: testScreenName,
      });
    const testProfilePubKeys = await testProfile.pubkeys();
    testProfilePDA = testProfilePubKeys.profile as anchor.web3.PublicKey;
    const testProfileTx = await testProfile.transaction();
    testProfileTx.recentBlockhash = (
      await rpcConnection.getLatestBlockhash()
    ).blockhash;
    testProfileTx.feePayer = testUser.publicKey;
    const signedTransaction = await testUserWallet.signTransaction(
      testProfileTx
    );
    await sendAndConfirmTransaction(rpcConnection, signedTransaction, [
      testUser,
    ]);
  });

  it("should create a connection", async () => {
    const connection = program.methods.createConnection().accounts({
      fromProfile: profilePDA,
      toProfile: testProfilePDA,
      user: userPDA,
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
      user: userPDA,
    });
    await connection.rpc();

    try {
      await program.account.connection.fetch(connectionPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist ${connectionPDA.toString()}`
      );
    }
  });
});
