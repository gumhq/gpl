import * as anchor from "@project-serum/anchor";

import {
  airdrop,
  gpl_core,
  gpl_compression,
  setupTree,
  to_leaf,
  assert_tree,
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

describe("Post Compression", async () => {
  let payer: Keypair;
  let merkleTree: PublicKey;
  let userPDA: PublicKey;
  let profilePDA: PublicKey;
  let treeConfigPDA: PublicKey;
  let offChainTree: MerkleTree;

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

    // Set up a profile
    const profileTx = gpl_core.methods
      .createProfile("Personal")
      .accounts({ user: userPDA, authority: payer.publicKey });
    profilePDA = (await profileTx.pubkeys()).profile;
    await profileTx.signers([payer]).rpc();
  });

  it("should create a compressed post", async () => {
    // increment the index
    const metadataUri = "https://www.example.com";
    const randomHash = randomBytes(32);
    await gpl_compression.methods
      .createCompressedPost(metadataUri, randomHash)
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
    const treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    assert_tree(treeData, offChainTree);

    const postSeeds = [Buffer.from("post"), randomHash];

    const post = {
      metadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    const postLeaf = await to_leaf(merkleTree, "Post", post, postSeeds);
    offChainTree.updateLeaf(0, postLeaf);
  });

  it("should create and update a compressed post", async () => {
    // increment the index
    let treeData: any;
    const metadataUri = "http://example.com";
    const randomHash = randomBytes(32);
    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    await gpl_compression.methods
      .createCompressedPost(metadataUri, randomHash)
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

    const postSeeds = [Buffer.from("post"), randomHash];

    const oldPost = {
      metadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    let index = 0;
    const oldPostLeaf = await to_leaf(merkleTree, "Post", oldPost, postSeeds);
    offChainTree.updateLeaf(index, oldPostLeaf);

    const newMetadataUri = "http://example1.com";

    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    const proof = offChainTree.getProof(index);
    const remainingAccounts = proof.proof.map((p) => {
      return { pubkey: new PublicKey(p), isWritable: false, isSigner: false };
    });

    await gpl_compression.methods
      .updateCompressedPost(
        // @ts-ignore
        metadataUri,
        newMetadataUri,
        randomHash,
        proof.root,
        index
      )
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
      .remainingAccounts(remainingAccounts)
      .signers([payer])
      .rpc();
    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    assert_tree(treeData, offChainTree);

    const newPost = {
      metadataUri: newMetadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    const newPostLeaf = await to_leaf(merkleTree, "Post", newPost, postSeeds);
    offChainTree.updateLeaf(index, newPostLeaf);
  });

  it("should create and delete a compressed post", async () => {
    // increment the index
    let treeData: any;
    const metadataUri = "http://example.com";
    const randomHash = randomBytes(32);
    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    await gpl_compression.methods
      //@ts-ignore
      .createCompressedPost(metadataUri, randomHash)
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

    assert_tree(treeData, offChainTree);

    const postSeeds = [Buffer.from("post"), randomHash];

    const oldPost = {
      metadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    let index = 0;
    const oldPostLeaf = await to_leaf(merkleTree, "Post", oldPost, postSeeds);
    offChainTree.updateLeaf(index, oldPostLeaf);

    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    const proof = offChainTree.getProof(index);
    const remainingAccounts = proof.proof.map((p) => {
      return { pubkey: new PublicKey(p), isWritable: false, isSigner: false };
    });

    await gpl_compression.methods
      .deleteCompressedPost(
        // @ts-ignore
        metadataUri,
        randomHash,
        proof.root,
        index
      )
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
      .remainingAccounts(remainingAccounts)
      .signers([payer])
      .rpc();
    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    assert_tree(treeData, offChainTree);

    const newPostLeaf = Buffer.from(Array(32).fill(0));
    offChainTree.updateLeaf(index, newPostLeaf);
  });
});
