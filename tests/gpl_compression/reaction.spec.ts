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
    // Kinda hack but this should be a compressed post
    const postRandomHash = randomBytes(32);
    const metadataUri = "https://example.com";
    const post = gpl_core.methods
      .createPost(metadataUri, postRandomHash)
      .accounts({
        user: userPDA,
        profile: profilePDA,
        authority: payer.publicKey,
      });
    const postPubKeys = await post.pubkeys();
    postPDA = postPubKeys.post as anchor.web3.PublicKey;
    await post.signers([payer]).rpc();
  });

  it("should create a compressed reaction", async () => {
    await gpl_compression.methods
      .createCompressedReaction(postPDA, "Haha")
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

    const reactionSeeds = [
      Buffer.from("reaction"),
      Buffer.from("Haha"),
      postPDA.toBuffer(),
      profilePDA.toBuffer(),
    ];

    const reaction = {
      fromProfile: profilePDA,
      toPost: postPDA,
      // Weird anchor trick for passing enums
      reactionType: { haha: {} },
    };

    const reactionLeaf = await to_leaf(
      merkleTree,
      "Reaction",
      reaction,
      reactionSeeds
    );

    offChainTree.updateLeaf(0, reactionLeaf);
  });

  it("should create and delete a compressed reaction", async () => {
    let index = 0;
    await gpl_compression.methods
      .createCompressedReaction(postPDA, "Haha")
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

    const reactionSeeds = [
      Buffer.from("reaction"),
      Buffer.from("Haha"),
      postPDA.toBuffer(),
      profilePDA.toBuffer(),
    ];

    const reaction = {
      fromProfile: profilePDA,
      toPost: postPDA,
      // Weird anchor trick for passing enums
      reactionType: { haha: {} },
    };

    const reactionLeaf = await to_leaf(
      merkleTree,
      "Reaction",
      reaction,
      reactionSeeds
    );

    offChainTree.updateLeaf(index, reactionLeaf);

    const proof = offChainTree.getProof(index);
    const remainingAccounts = proof.proof.map((p) => {
      return { pubkey: new PublicKey(p), isWritable: false, isSigner: false };
    });

    await gpl_compression.methods
      .deleteCompressedReaction(postPDA, "Haha", proof.root, index)
      .accounts({
        user: userPDA,
        fromProfile: profilePDA,
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
