import * as anchor from "@project-serum/anchor";
import randombytes from "randombytes";
import { expect } from "chai";
import { GplCompression } from "../../target/types/gpl_compression";
import { airdrop } from "../utils/index";
import {
  getConcurrentMerkleTreeAccountSize,
  createVerifyLeafIx,
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
  ValidDepthSizePair,
} from "@solana/spl-account-compression";

import {
  Keypair,
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

const program = anchor.workspace
  .GplCompression as anchor.Program<GplCompression>;

anchor.setProvider(anchor.AnchorProvider.env());
const rpcConnection = anchor.getProvider().connection;

async function setupTree(
  payer: Keypair,
  depthSizePair: ValidDepthSizePair,
  connection: Connection
): Promise<PublicKey> {
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

  await program.methods
    .initialize(depthSizePair.maxDepth, depthSizePair.maxBufferSize)
    .accounts({
      authority: payer.publicKey,
      merkleTree: merkleTree,
      logWrapper: SPL_NOOP_PROGRAM_ID,
      compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
    })
    .preInstructions([allocTreeIx])
    .signers([payer, merkleTreeKeypair])
    .rpc();
  return merkleTree;
}

describe("Compression", async () => {
  const payer = anchor.web3.Keypair.generate();
  before(async () => {
    await airdrop(payer.publicKey);
  });

  it("should do something", async () => {
    const merkleTree = await setupTree(
      payer,
      {
        maxDepth: 14,
        maxBufferSize: 64,
      },
      rpcConnection
    );

    console.log("merkleTree", merkleTree.toString());
    const accountInfo = await rpcConnection.getAccountInfo(merkleTree);
    const merkleTreeData = ConcurrentMerkleTreeAccount.fromBuffer(
      accountInfo!.data!
    );
    console.log(merkleTreeData);
  });
});
