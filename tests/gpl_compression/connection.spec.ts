import * as anchor from "@project-serum/anchor";

import {
  airdrop,
  gpl_core,
  gpl_compression,
  setupTree,
  to_leaf,
  createGumTld,
  createGumDomain,
} from "../utils/index";

import {
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
  MerkleTree,
} from "@solana/spl-account-compression";

import { Keypair, PublicKey } from "@solana/web3.js";

import { expect } from "chai";
import randomBytes from "randombytes";

import { faker } from "@faker-js/faker";

anchor.setProvider(anchor.AnchorProvider.env());
const rpcConnection = anchor.getProvider().connection;

describe("Connection Compression", async () => {
  let payer: Keypair;
  let merkleTree: PublicKey;
  let userPDA: PublicKey;
  let profilePDA: PublicKey;
  let testProfilePDA: anchor.web3.PublicKey;
  let treeConfigPDA: PublicKey;
  let offChainTree: MerkleTree;
  let testUser: Keypair;

  beforeEach(async () => {
    // Setup a new keypair and airdrop some SOL
    payer = anchor.web3.Keypair.generate();
    await airdrop(payer.publicKey);
    const treeResult = await setupTree(
      payer,
      {
        maxDepth: 14,
        maxBufferSize: 64,
      },
      rpcConnection
    );
    merkleTree = treeResult.merkleTree;
    treeConfigPDA = treeResult.treeConfigPDA;
    offChainTree = treeResult.offChainTree;

    // Set up a user
    const randomHash = randomBytes(32);
    const userTx = gpl_core.methods.createUser(randomHash).accounts({
      authority: payer.publicKey,
    });
    userPDA = (await userTx.pubkeys()).user;
    await userTx.signers([payer]).rpc();

    const gumTld = await createGumTld();

    // Set up a profile
    const profileMetdataUri = "https://example.com";
    const screenName = await createGumDomain(
      gumTld,
      faker.internet.userName(),
      payer
    );
    const profileTx = gpl_core.methods
      .createProfile("Personal", profileMetdataUri)
      .accounts({ user: userPDA, authority: payer.publicKey, screenName });
    profilePDA = (await profileTx.pubkeys()).profile;
    await profileTx.signers([payer]).rpc();

    // Create a testUser
    testUser = anchor.web3.Keypair.generate();
    await airdrop(testUser.publicKey);

    const randomTestHash = randomBytes(32);
    const createTestUser = gpl_core.methods
      .createUser(randomTestHash)
      .accounts({ authority: testUser.publicKey });
    const testUserPubKeys = await createTestUser.pubkeys();
    let testUserPDA = testUserPubKeys.user as anchor.web3.PublicKey;

    await createTestUser.signers([testUser]).rpc();

    // Create a testProfile
    const testProfileMetdataUri = "https://example.com";
    const testScreenName = await createGumDomain(
      gumTld,
      faker.internet.userName(),
      testUser
    );
    const testProfile = gpl_core.methods
      .createProfile("Personal", testProfileMetdataUri)
      .accounts({
        user: testUserPDA,
        authority: testUser.publicKey,
        screenName: testScreenName,
      });
    const testProfilePubKeys = await testProfile.pubkeys();
    testProfilePDA = testProfilePubKeys.profile as anchor.web3.PublicKey;

    await testProfile.signers([testUser]).rpc();
  });

  it("should create a compressed connection", async () => {
    // increment the index
    const randomHash = randomBytes(32);
    await gpl_compression.methods
      .createCompressedConnection()
      .accounts({
        user: userPDA,
        fromProfile: profilePDA,
        toProfile: testProfilePDA,
        treeConfig: treeConfigPDA,
        merkleTree,
        authority: payer.publicKey,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        logWrapperProgram: SPL_NOOP_PROGRAM_ID,
        gplCoreProgram: gpl_core.programId,
      })
      .signers([payer])
      .rpc();
    const treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );
    expect(treeData).to.not.be.null;

    const connectionSeeds = [
      Buffer.from("connection"),
      profilePDA.toBuffer(),
      testProfilePDA.toBuffer(),
    ];

    const connection = {
      fromProfile: profilePDA,
      toProfile: testProfilePDA,
    };

    const connectionLeaf = await to_leaf(
      merkleTree,
      "Connection",
      connection,
      connectionSeeds
    );
    offChainTree.updateLeaf(0, connectionLeaf);
  });

  it("should create and delete a compressed connection", async () => {
    let index = 0;
    const randomHash = randomBytes(32);
    await gpl_compression.methods
      .createCompressedConnection()
      .accounts({
        user: userPDA,
        fromProfile: profilePDA,
        toProfile: testProfilePDA,
        treeConfig: treeConfigPDA,
        merkleTree,
        authority: payer.publicKey,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        logWrapperProgram: SPL_NOOP_PROGRAM_ID,
        gplCoreProgram: gpl_core.programId,
      })
      .signers([payer])
      .rpc();
    const treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );
    expect(treeData).to.not.be.null;

    const connectionSeeds = [
      Buffer.from("connection"),
      profilePDA.toBuffer(),
      testProfilePDA.toBuffer(),
    ];

    const connection = {
      fromProfile: profilePDA,
      toProfile: testProfilePDA,
    };

    const oldConnectionLeaf = await to_leaf(
      merkleTree,
      "Connection",
      connection,
      connectionSeeds
    );
    offChainTree.updateLeaf(index, oldConnectionLeaf);

    const proof = offChainTree.getProof(index);
    const remainingAccounts = proof.proof.map((p) => {
      return { pubkey: new PublicKey(p), isWritable: false, isSigner: false };
    });

    await gpl_compression.methods
      .deleteCompressedConnection(proof.root, index)
      .accounts({
        user: userPDA,
        fromProfile: profilePDA,
        toProfile: testProfilePDA,
        treeConfig: treeConfigPDA,
        merkleTree,
        authority: payer.publicKey,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        logWrapperProgram: SPL_NOOP_PROGRAM_ID,
        gplCoreProgram: gpl_core.programId,
      })
      .remainingAccounts(remainingAccounts)
      .signers([payer])
      .rpc();

    const newConnectionLeaf = Buffer.from(Array(32).fill(0));
    offChainTree.updateLeaf(index, newConnectionLeaf);
  });
});
