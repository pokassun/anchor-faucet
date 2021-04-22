import { BN, Provider, setProvider, web3, workspace } from "@project-serum/anchor";
import {
  createMint,
  createTokenAccount,
  getTokenAccount,
} from "@project-serum/common";
import { TokenInstructions } from "@project-serum/serum";
import assert from 'assert';

describe("Faucet Test", () => {
  
  const provider = Provider.local();
  setProvider(provider);
  
  // Add your test here.
  const program = workspace.Faucet;

  const bobFaucet = new web3.Account();

  let mint: web3.PublicKey;
  let mintAuth: web3.PublicKey;

  it("Initialized bobFaucet", async () => {

    const [_mintAuth, nonce] = await web3.PublicKey.findProgramAddress(
      [bobFaucet.publicKey.toBuffer()],
      program.programId
    );

    mintAuth= _mintAuth;
    mint = await createMint(provider, mintAuth, 8);

    await program.rpc.initialize(nonce, {
      accounts: {
        faucet: bobFaucet.publicKey,
        mint,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      },
      instructions: [await program.account.faucet.createInstruction(bobFaucet)],
      signers: [bobFaucet],
    });

  });

  let bob: web3.PublicKey;
  let bobWallet: web3.Account;
  it("Give bob 10 coins", async () => {
    bobWallet = new web3.Account();
    bob = await createTokenAccount(
      provider,
      mint,
      bobWallet.publicKey
    );
    await program.rpc.drip({
      accounts: {
        faucet: bobFaucet.publicKey,
        mint,
        mintAuth,
        receiver: bob,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
    const bobBalance = await getTokenAccount(provider, bob);
    assert.ok(bobBalance.amount.eq(new BN(10)))
  });

  it("Transfer 10 coins from Bob to Alice", async () => {
    const aliceFaucet = new web3.Account();
    const [checkSigner, nonce] = await web3.PublicKey.findProgramAddress(
      [aliceFaucet.publicKey.toBuffer()],
      program.programId
    );

    await program.rpc.initialize(nonce, {
      accounts: {
        faucet: aliceFaucet.publicKey,
        mint: new web3.Account().publicKey,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      },
      instructions: [await program.account.faucet.createInstruction(aliceFaucet)],
      signers: [aliceFaucet],
    });

    const aliceWallet = new web3.Account();
    const alice = await createTokenAccount(
      provider,
      mint,
      aliceWallet.publicKey,
    );

    await program.rpc.transfer(nonce, {
      accounts: {
        faucet: aliceFaucet.publicKey,
        to: alice,
        from: bob,
        owner: checkSigner,
        checkSigner,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
    const bobBalance = await getTokenAccount(provider, bob);
    const aliceBalance = await getTokenAccount(provider, alice);
    assert.ok(bobBalance.amount.eq(new BN(0)))
    assert.ok(aliceBalance.amount.eq(new BN(10)))
  });
});
