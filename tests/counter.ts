import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Counter } from "../target/types/counter";
import { expect } from "chai";
describe("counter", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
  
    const program = anchor.workspace.Counter as Program<Counter>;
    const provider = anchor.getProvider();
  
    // Generate a new keypair for the counter account
    const counterKeypair = anchor.web3.Keypair.generate();
  
    it("Initialize counter", async () => {
      const tx = await program.methods
        .initialize()
        .accounts({
          counter: counterKeypair.publicKey,
          user: provider.wallet.publicKey,
        })
        .signers([counterKeypair])
        .rpc();
  
      console.log("Initialize transaction signature:", tx);
  
      // Fetch the counter account
      const counterAccount = await program.account.counter.fetch(
        counterKeypair.publicKey
      );
  
      expect(counterAccount.count.toNumber()).to.equal(0);
      expect(counterAccount.authority.toString()).to.equal(
        provider.wallet.publicKey.toString()
      );
    });
  
    it("Increment counter", async () => {
      const tx = await program.methods
        .increment()
        .accounts({
          counter: counterKeypair.publicKey,
        })
        .rpc();
  
      console.log("Increment transaction signature:", tx);
  
      const counterAccount = await program.account.counter.fetch(
        counterKeypair.publicKey
      );
  
      expect(counterAccount.count.toNumber()).to.equal(1);
    });
  
    it("Increment counter again", async () => {
      await program.methods
        .increment()
        .accounts({
          counter: counterKeypair.publicKey,
        })
        .rpc();
  
      const counterAccount = await program.account.counter.fetch(
        counterKeypair.publicKey
      );
  
      expect(counterAccount.count.toNumber()).to.equal(2);
    });
  
    it("Decrement counter", async () => {
      const tx = await program.methods
        .decrement()
        .accounts({
          counter: counterKeypair.publicKey,
        })
        .rpc();
  
      console.log("Decrement transaction signature:", tx);
  
      const counterAccount = await program.account.counter.fetch(
        counterKeypair.publicKey
      );
  
      expect(counterAccount.count.toNumber()).to.equal(1);
    });
  
    it("Multiple operations", async () => {
      // Increment 3 times
      for (let i = 0; i < 3; i++) {
        await program.methods
          .increment()
          .accounts({
            counter: counterKeypair.publicKey,
          })
          .rpc();
      }
  
      // Decrement 2 times
      for (let i = 0; i < 2; i++) {
        await program.methods
          .decrement()
          .accounts({
            counter: counterKeypair.publicKey,
          })
          .rpc();
      }
  
      const counterAccount = await program.account.counter.fetch(
        counterKeypair.publicKey
      );
  
      // Started at 1, +3, -2 = 2
      expect(counterAccount.count.toNumber()).to.equal(2);
    });
  });