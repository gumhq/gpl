import * as anchor from "@project-serum/anchor";

import {
  airdrop,
  gpl_core,
  gpl_compression,
  setupTree,
  to_leaf,
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

anchor.setProvider(anchor.AnchorProvider.env());
const rpcConnection = anchor.getProvider().connection;

describe("Reaction Compression", async () => {
  let payer: Keypair;
  let merkleTree: PublicKey;
  let treeConfigPDA: PublicKey;
  let offChainTree: MerkleTree;

  let userPDA: PublicKey;
  let profilePDA: PublicKey;
  let postPDA: PublicKey;
  let reactionPDA: PublicKey;

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

    const randomHash = randomBytes(32);
    const userTx = gpl_core.methods.createUser(randomHash).accounts({
      authority: payer.publicKey,
    });
    const userPubKeys = await userTx.pubkeys();
    userPDA = userPubKeys.user as anchor.web3.PublicKey;
    await userTx.signers([payer]).rpc();

    // Create a profile
    const profileTx = gpl_core.methods
      .createProfile("Personal")
      .accounts({ user: userPDA, authority: payer.publicKey });
    const profilePubKeys = await profileTx.pubkeys();
    profilePDA = profilePubKeys.profile as anchor.web3.PublicKey;
    await profileTx.signers([payer]).rpc();

    // Create a post
    const postRandomHash = randomBytes(32);
    const metadataUri = "https://example.com";
    const postSeeds = [Buffer.from("post"), randomHash];
    const [post, _] = await PublicKey.findProgramAddress(
      postSeeds,
      gpl_core.programId
    );

    postPDA = post;

    await gpl_compression.methods
      .createCompressedPost(metadataUri, postRandomHash)
      .accounts({
        user: userPDA,
        profile: profilePDA,
        treeConfig: treeConfigPDA,
        merkleTree,
        authority: payer.publicKey,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        logWrapperProgram: SPL_NOOP_PROGRAM_ID,
        gplCoreProgram: gpl_core.programId,
      })
      .signers([payer])
      .rpc();
  });

  it("should create a compressed comment", async () => {
    const randomHash = randomBytes(32);
    const metadataUri = "https://example.com";
    await gpl_compression.methods
      .createCompressedComment(postPDA, metadataUri, randomHash)
      .accounts({
        user: userPDA,
        fromProfile: profilePDA,
        treeConfig: treeConfigPDA,
        merkleTree,
        targetTreeConfig: treeConfigPDA,
        targetMerkleTree: merkleTree,
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

    const postSeeds = [Buffer.from("post"), randomHash];

    const post = {
      metadata_uri: metadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: postPDA,
    };

    const commentLeaf = await to_leaf(merkleTree, "Post", post, postSeeds);

    offChainTree.updateLeaf(0, commentLeaf);
  });
});
