import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GplCore } from "../target/types/gpl_core";

describe("gpl core", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GplCore as Program<GplCore>;

  it("Is initialized!", async () => {
    console.log("test");
  });
});
