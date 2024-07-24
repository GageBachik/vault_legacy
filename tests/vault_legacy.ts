import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
// import { VaultLegacy } from "../target/types/vault_legacy";

describe("vault_legacy", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  // const program = anchor.workspace.VaultLegacy as Program<VaultLegacy>;

  it("Is initialized!", async () => {
    // // Add your test here.
    // const tx = await program.methods.initialize().rpc();
    // console.log("Your transaction signature", tx);
    let testKey = anchor.web3.Keypair.generate();
    console.log("testKey", testKey.publicKey.toString());
  });
});
