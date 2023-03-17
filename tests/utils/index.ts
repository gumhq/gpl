import * as anchor from "@project-serum/anchor";
import {
  PublicKey,
  LAMPORTS_PER_SOL,
  Keypair,
  SystemProgram,
  Connection,
} from "@solana/web3.js";

import { GplCompression } from "../../target/types/gpl_compression";
import { GplCore } from "../../target/types/gpl_core";
import { GplNameservice } from "../../target/types/gpl_nameservice";
import pkg from "js-sha3";

import {
  getConcurrentMerkleTreeAccountSize,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
  ValidDepthSizePair,
  MerkleTree,
  ConcurrentMerkleTreeAccount,
} from "@solana/spl-account-compression";

const { keccak_256 } = pkg;

const provider = anchor.getProvider();

export const gpl_core = anchor.workspace.GplCore as anchor.Program<GplCore>;
export const gpl_compression = anchor.workspace
  .GplCompression as anchor.Program<GplCompression>;
export const gpl_nameservice = anchor.workspace
  .GplNameservice as anchor.Program<GplNameservice>;

// keccak256 hash of "gum"
const gumTldHash = keccak_256("gum");

const [gumTld, _] = anchor.web3.PublicKey.findProgramAddressSync(
  [
    Buffer.from("name_record"),
    Buffer.from(gumTldHash, "hex"),
    anchor.web3.PublicKey.default.toBuffer(),
  ],
  gpl_nameservice.programId
);

export async function createGumTld(): Promise<PublicKey> {
  try {
    await gpl_nameservice.account.nameRecord.fetch(gumTld);
  } catch (error: any) {
    await gpl_nameservice.methods
      .createTld("gum")
      .accounts({
        nameRecord: gumTld,
      })
      .rpc();
  }
  return gumTld;
}

export async function createGumDomain(
  gumTld: PublicKey,
  domain: string,
  owner?: Keypair
): Promise<PublicKey> {
  // keccak256 hash of domain
  const domainHash = keccak_256(domain);
  const [nameRecord, _] = await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("name_record"),
      Buffer.from(domainHash, "hex"),
      gumTld.toBuffer(),
    ],
    gpl_nameservice.programId
  );

  let signers: Keypair[] = [];

  if (owner !== undefined) {
    signers.push(owner);
  }

  await gpl_nameservice.methods
    .createNameRecord(domain)
    .accounts({
      domain: gumTld,
      nameRecord,
      // @ts-ignore
      authority: owner?.publicKey || provider.wallet.publicKey,
    })
    .signers(signers)
    .rpc();

  return nameRecord;
}

export function assert_tree(
  onChainTree: ConcurrentMerkleTreeAccount,
  offChainTree: MerkleTree
): boolean {
  const right = new PublicKey(onChainTree.getCurrentRoot()).toBase58();
  const left = new PublicKey(offChainTree.getRoot()).toBase58();
  return right === left;
}

export interface TreeInfo {
  merkleTree: PublicKey;
  treeConfigPDA: PublicKey;
  offChainTree: MerkleTree;
}

export async function airdrop(key: PublicKey) {
  const airdropSig = await provider.connection.requestAirdrop(
    key,
    1 * LAMPORTS_PER_SOL
  );
  return provider.connection.confirmTransaction(airdropSig);
}

export function hash(data: Buffer): Buffer {
  return Buffer.from(keccak_256.arrayBuffer(data));
}

async function find_asset_id(
  merkleTree: PublicKey,
  seedHash: Buffer
): Promise<PublicKey> {
  const [asset_id] = await PublicKey.findProgramAddress(
    [Buffer.from("asset"), merkleTree.toBuffer(), seedHash],
    gpl_compression.programId
  );
  return asset_id;
}

export async function to_leaf(
  merkleTree: PublicKey,
  name: any,
  data: any,
  seeds: Buffer[]
): Promise<Buffer> {
  const seedHash = hash(Buffer.concat(seeds));
  const assetId = await find_asset_id(merkleTree, seedHash);
  const dataSerialized = await gpl_core.coder.accounts.encode(name, data);
  const dataHash = hash(dataSerialized);
  const leaf = Buffer.concat([assetId.toBuffer(), seedHash, dataHash]);
  return hash(leaf);
}

export async function setupTree(
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

  const treeInitTx = gpl_compression.methods
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

  const offChainTree = new MerkleTree(leaves);

  return { merkleTree, treeConfigPDA, offChainTree };
}
