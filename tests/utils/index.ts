import * as anchor from '@project-serum/anchor';
import { PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';

const provider = anchor.getProvider();

export async function airdrop(key: PublicKey) {
  const airdropSig = await provider.connection.requestAirdrop(key, 1 * LAMPORTS_PER_SOL);
  return provider.connection.confirmTransaction(airdropSig);
}