import * as anchor from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import randombytes from "randombytes";
import { airdrop } from "../utils";
import { expect } from "chai";
import { sendAndConfirmTransaction } from "@solana/web3.js";
import { GplCore } from "../../target/types/gpl_core";

const program = anchor.workspace.GplCore as anchor.Program<GplCore>;

anchor.setProvider(anchor.AnchorProvider.env());
const user = (anchor.getProvider() as any).wallet.payer;

describe("User", async () => {
  let connection: anchor.web3.Connection;
  let userPDA: anchor.web3.PublicKey;
  let randomUser: anchor.web3.Keypair;

  before(async () => {
    randomUser = anchor.web3.Keypair.generate();
    await airdrop(randomUser.publicKey);
    connection = new anchor.web3.Connection(
      "http://localhost:8899",
      "confirmed"
    );
  });

  it("should create a user", async () => {
    const randomHash = randombytes(32);
    const tx = program.methods.createUser(randomHash);
    const pubKeys = await tx.pubkeys();
    userPDA = pubKeys.user as anchor.web3.PublicKey;
    await tx.rpc();
    const userAccount = await program.account.user.fetch(userPDA);
    expect(userAccount.authority.toString()).is.equal(
      user.publicKey.toString()
    );
  });

  it("should update a user", async () => {
    const tx = program.methods
      .updateUser()
      .accounts({ user: userPDA, newAuthority: randomUser.publicKey });
    const pubKeys = await tx.pubkeys();
    const randomUserPDA = pubKeys.user as anchor.web3.PublicKey;
    await tx.rpc();
    const userAccount = await program.account.user.fetch(randomUserPDA);
    expect(userAccount.authority.toString()).is.equal(
      randomUser.publicKey.toString()
    );
  });

  it("should delete a user", async () => {
    const randomUserWallet = new NodeWallet(randomUser);
    const tx = program.methods
      .deleteUser()
      .accounts({ user: userPDA, authority: randomUser.publicKey });
    const pubKeys = await tx.pubkeys();
    const randomUserPDA = pubKeys.user as anchor.web3.PublicKey;
    const transaction = await tx.transaction();
    transaction.recentBlockhash = (
      await connection.getLatestBlockhash()
    ).blockhash;
    transaction.feePayer = randomUser.publicKey;
    const signedTransaction = await randomUserWallet.signTransaction(
      transaction
    );
    await sendAndConfirmTransaction(connection, signedTransaction, [
      randomUser,
    ]);
    try {
      await program.account.user.fetch(randomUserPDA);
    } catch (error: any) {
      expect(error).to.be.an("error");
      expect(error.toString()).to.contain(
        `Account does not exist ${randomUserPDA.toString()}`
      );
    }
  });
});

