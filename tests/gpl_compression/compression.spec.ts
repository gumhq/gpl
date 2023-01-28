import * as anchor from "@project-serum/anchor";
import { GplCompression } from "../../target/types/gpl_compression";
import { airdrop, gpl_core, hash } from "../utils/index";
import {
  getConcurrentMerkleTreeAccountSize,
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
  ValidDepthSizePair,
  MerkleTree,
  MerkleTreeProof,
} from "@solana/spl-account-compression";

import { Keypair, Connection, PublicKey, SystemProgram } from "@solana/web3.js";

import { expect } from "chai";
import randomBytes from "randombytes";

const program = anchor.workspace
  .GplCompression as anchor.Program<GplCompression>;

anchor.setProvider(anchor.AnchorProvider.env());
const rpcConnection = anchor.getProvider().connection;

interface TreeInfo {
  merkleTree: PublicKey;
  treeConfigPDA: PublicKey;
  offChainTree: MerkleTree;
}

async function setupTree(
  payer: Keypair,
  depthSizePair: ValidDepthSizePair,
  connection: Connection
): Promise<TreeInfo> {
  const merkleTreeKeypair = Keypair.generate();
  const merkleTree = merkleTreeKeypair.publicKey;
  const space = getConcurrentMerkleTreeAccountSize(
    depthSizePair.maxDepth,
    depthSizePair.maxBufferSize
  );

  const allocTreeIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: merkleTree,
    lamports: await connection.getMinimumBalanceForRentExemption(space),
    space: space,
    programId: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  });

  const treeInitTx = program.methods
    .initializeTree(depthSizePair.maxDepth, depthSizePair.maxBufferSize)
    .accounts({
      authority: payer.publicKey,
      merkleTree: merkleTree,
      logWrapper: SPL_NOOP_PROGRAM_ID,
      compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
    });

  const treeConfigPDA = (await treeInitTx.pubkeys()).treeConfig;
  await treeInitTx
    .preInstructions([allocTreeIx])
    .signers([payer, merkleTreeKeypair])
    .rpc();

  const leaves = Array(2 ** depthSizePair.maxDepth).fill(Buffer.alloc(32));
  // for (let i = 0; i < leaves.length; i++) {
  //   leaves[i] = randomBytes(32);
  // }

  const offChainTree = new MerkleTree(leaves);

  return { merkleTree, treeConfigPDA, offChainTree };
}

async function find_asset_id(
  merkleTree: PublicKey,
  seedHash: Buffer
): Promise<PublicKey> {
  const [asset_id] = await PublicKey.findProgramAddress(
    [Buffer.from("asset"), merkleTree.toBuffer(), seedHash],
    SPL_ACCOUNT_COMPRESSION_PROGRAM_ID
  );
  return asset_id;
}

async function to_leaf(
  merkleTree: PublicKey,
  name: any,
  data: any,
  seeds: Buffer[]
): Promise<Buffer> {
  const seedHash = hash(Buffer.concat(seeds));
  const assetId = await find_asset_id(merkleTree, seedHash);
  const dataSerialized = await gpl_core.coder.accounts.encode(name, data);
  const dataHash = hash(dataSerialized);
  const leaf = [assetId.toBuffer(), seedHash, dataHash];
  return hash(Buffer.concat(leaf));
}

describe("Compression SerDe", () => {
  it("should serialize", async () => {
    const merkleTree = Keypair.generate().publicKey;

    const post = {
      metadata_uri: "https://www.google.com",
      randomHash: randomBytes(32),
      profile: Keypair.generate().publicKey,
      reply_to: null,
    };

    const postSeeds = [Buffer.from("post"), post.randomHash];
    const node = await to_leaf(merkleTree, "Post", post, postSeeds);
    expect(node).to.not.be.null;
  });
});

describe("Compression", async () => {
  const payer = anchor.web3.Keypair.generate();
  before(async () => {
    await airdrop(payer.publicKey);
  });

  it("should set up the tree", async () => {
    const { merkleTree } = await setupTree(
      payer,
      {
        maxDepth: 14,
        maxBufferSize: 64,
      },
      rpcConnection
    );

    const merkleTreeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    expect(merkleTreeData).to.not.be.null;
  });
});

describe("GPL Compression", async () => {
  let payer: Keypair;
  let merkleTree: PublicKey;
  let userPDA: PublicKey;
  let profilePDA: PublicKey;
  let treeConfigPDA: PublicKey;
  let offChainTree: MerkleTree;

  beforeEach(async () => {
    // Setup a new keypair and airdrop some SOL
    payer = anchor.web3.Keypair.generate();
    console.log("payer:", payer.publicKey.toBase58());
    await airdrop(payer.publicKey);
    console.log("Airdrop Complete");
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

    console.log("Merkle Tree:", merkleTree.toBase58());
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
    await program.methods
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
    expect(treeData).to.not.be.null;
    const postSeeds = [Buffer.from("post"), randomHash];

    const post = {
      metadata_uri: metadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    const postLeaf = await to_leaf(merkleTree, "Post", post, postSeeds);
    offChainTree.updateLeaf(0, postLeaf);
  });

  it("should create and update compressed post", async () => {
    // increment the index
    let treeData: any;
    const metadataUri = "http://example.com";
    const randomHash = randomBytes(32);
    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    await program.methods
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
      metadata_uri: metadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    let index = 0;
    const oldPostLeaf = await to_leaf(merkleTree, "Post", oldPost, postSeeds);
    offChainTree.updateLeaf(index, oldPostLeaf);

    console.log(offChainTree.getRoot().toString("hex"));

    const newMetadataUri = "http://example1.com";

    treeData = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      rpcConnection,
      merkleTree
    );

    const proof = offChainTree.getProof(index);
    const remainingAccounts = proof.proof.map((p) => {
      return { pubkey: new PublicKey(p), isWritable: false, isSigner: false };
    });

    await program.methods
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
    expect(treeData).to.not.be.null;
    const newPost = {
      metadataUri: newMetadataUri,
      randomHash,
      profile: profilePDA,
      replyTo: null,
    };

    const newPostLeaf = await to_leaf(merkleTree, "Post", newPost, postSeeds);
    offChainTree.updateLeaf(index, newPostLeaf);
  });
});
