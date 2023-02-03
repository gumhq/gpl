import * as anchor from "@project-serum/anchor";
import { airdrop, setupTree, to_leaf } from "../utils/index";
import { ConcurrentMerkleTreeAccount } from "@solana/spl-account-compression";

import { Keypair } from "@solana/web3.js";

import { expect } from "chai";
import randomBytes from "randombytes";

anchor.setProvider(anchor.AnchorProvider.env());
const rpcConnection = anchor.getProvider().connection;

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
