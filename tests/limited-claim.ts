import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LimitedClaim } from "../target/types/limited_claim";

describe("limited_claim", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.limitedClaim as Program<LimitedClaim>;
  const provider = anchor.AnchorProvider.env();

  const admin = provider.wallet;
  const claimer = anchor.web3.Keypair.generate();

  let counterPda: anchor.web3.PublicKey;
  let counterBump: number;

  it("Initialize counter", async () => {
    [counterPda, counterBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("counter"), admin.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .initializeCounter(new anchor.BN(3), new anchor.BN(0))
      .accounts({
        counter: counterPda,
        admin: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

  });

  it("Claim a seat", async () => {
    const [receiptPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), counterPda.toBuffer(), claimer.publicKey.toBuffer()],
      program.programId
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(claimer.publicKey, 1_000_000_000) // 1 SOL
    );

    const tx = await program.methods
      .claim()
      .accounts({
        counter: counterPda,
        receipt: receiptPda,
        claimer: claimer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([claimer])
      .rpc();

  });

  it("Cancel claim", async () => {
    const [receiptPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), counterPda.toBuffer(), claimer.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .cancel()
      .accounts({
        counter: counterPda,
        receipt: receiptPda,
        claimer: claimer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([claimer])
      .rpc();
  });
});
