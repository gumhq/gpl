import * as anchor from "@project-serum/anchor";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

import { GplCompression } from "../../target/types/gpl_compression";
import { GplCore } from "../../target/types/gpl_core";
import pkg from "js-sha3";

const { keccak_256 } = pkg;

const provider = anchor.getProvider();

export const gpl_core = anchor.workspace.GplCore as anchor.Program<GplCore>;

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

