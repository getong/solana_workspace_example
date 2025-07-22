import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PdaLimitation } from "../target/types/pda_limitation";
import { expect } from "chai";

describe("pda_limitation", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PdaLimitation as Program<PdaLimitation>;
  const provider = anchor.getProvider();
  const user = (provider.wallet as anchor.Wallet).payer;

  let todoPda: anchor.web3.PublicKey;
  let todoBump: number;
  let globalCounterPda: anchor.web3.PublicKey;
  let globalCounterBump: number;

  before(async () => {
    [todoPda, todoBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("TODO_ACC"), user.publicKey.toBuffer()],
      program.programId
    );

    [globalCounterPda, globalCounterBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("GLOBAL_TODO_COUNTER")],
      program.programId
    );
  });

  it("Initialize global counter", async () => {
    const tx = await program.methods
      .initializeGlobalCounter()
      .accounts({
        signer: user.publicKey,
        globalCounter: globalCounterPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Initialize global counter transaction signature", tx);

    const globalCounterAccount = await program.account.globalTodoCounter.fetch(globalCounterPda);
    expect(globalCounterAccount.totalTodos.toNumber()).to.equal(0);
  });

  it("Initialize user PDA", async () => {
    const tx = await program.methods
      .initializePda()
      .accounts({
        signer: user.publicKey,
        todoAccount: todoPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Initialize PDA transaction signature", tx);

    const todoAccount = await program.account.todoState.fetch(todoPda);
    expect(todoAccount.key.toString()).to.equal(user.publicKey.toString());
    expect(todoAccount.totalTodos.toNumber()).to.equal(0);
    expect(todoAccount.todos.length).to.equal(0);
  });

  it("Add a todo", async () => {
    const title = "Test Todo";
    const description = "This is a test todo item";

    const tx = await program.methods
      .addTodo(title, description)
      .accounts({
        signer: user.publicKey,
        todoAccount: todoPda,
        globalCounter: globalCounterPda,
      })
      .signers([user])
      .rpc();

    console.log("Add todo transaction signature", tx);

    const todoAccount = await program.account.todoState.fetch(todoPda);
    expect(todoAccount.todos.length).to.equal(1);
    expect(todoAccount.todos[0].title).to.equal(title);
    expect(todoAccount.todos[0].description).to.equal(description);
    expect(todoAccount.todos[0].isCompleted).to.be.false;
    expect(todoAccount.totalTodos.toNumber()).to.equal(1);

    // Check global counter
    const globalCounterAccount = await program.account.globalTodoCounter.fetch(globalCounterPda);
    expect(globalCounterAccount.totalTodos.toNumber()).to.equal(1);
  });

  it("Add another todo", async () => {
    const title = "Second Todo";
    const description = "This is the second todo";

    const tx = await program.methods
      .addTodo(title, description)
      .accounts({
        signer: user.publicKey,
        todoAccount: todoPda,
        globalCounter: globalCounterPda,
      })
      .signers([user])
      .rpc();

    console.log("Add second todo transaction signature", tx);

    const todoAccount = await program.account.todoState.fetch(todoPda);
    expect(todoAccount.todos.length).to.equal(2);
    expect(todoAccount.totalTodos.toNumber()).to.equal(2);

    // Check global counter increased
    const globalCounterAccount = await program.account.globalTodoCounter.fetch(globalCounterPda);
    expect(globalCounterAccount.totalTodos.toNumber()).to.equal(2);
  });

  it("Update a todo", async () => {
    const tx = await program.methods
      .updateTodo(new anchor.BN(0), true)
      .accounts({
        signer: user.publicKey,
        todoAccount: todoPda,
      })
      .signers([user])
      .rpc();

    console.log("Update todo transaction signature", tx);

    const todoAccount = await program.account.todoState.fetch(todoPda);
    expect(todoAccount.todos[0].isCompleted).to.be.true;
  });

  it("Remove a todo", async () => {
    const tx = await program.methods
      .removeTodo(new anchor.BN(0))
      .accounts({
        signer: user.publicKey,
        todoAccount: todoPda,
      })
      .signers([user])
      .rpc();

    console.log("Remove todo transaction signature", tx);

    const todoAccount = await program.account.todoState.fetch(todoPda);
    expect(todoAccount.todos.length).to.equal(1);
    expect(todoAccount.todos[0].title).to.equal("Second Todo");
  });

  it("Test with multiple users", async () => {
    // Create a new user
    const newUser = anchor.web3.Keypair.generate();
    
    // Airdrop SOL to new user
    const airdropSignature = await provider.connection.requestAirdrop(
      newUser.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);

    // Find PDA for new user
    const [newUserTodoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("TODO_ACC"), newUser.publicKey.toBuffer()],
      program.programId
    );

    // Initialize PDA for new user
    await program.methods
      .initializePda()
      .accounts({
        signer: newUser.publicKey,
        todoAccount: newUserTodoPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([newUser])
      .rpc();

    // Add todo for new user
    await program.methods
      .addTodo("New User Todo", "Todo from a different user")
      .accounts({
        signer: newUser.publicKey,
        todoAccount: newUserTodoPda,
        globalCounter: globalCounterPda,
      })
      .signers([newUser])
      .rpc();

    // Check global counter includes todos from both users
    const globalCounterAccount = await program.account.globalTodoCounter.fetch(globalCounterPda);
    expect(globalCounterAccount.totalTodos.toNumber()).to.equal(3); // 2 from first user + 1 from new user
  });

  it("Test error cases", async () => {
    // Test title too long
    try {
      await program.methods
        .addTodo("A".repeat(51), "Description")
        .accounts({
          signer: user.publicKey,
          todoAccount: todoPda,
          globalCounter: globalCounterPda,
        })
        .signers([user])
        .rpc();
      expect.fail("Should have failed with title too long");
    } catch (error) {
      expect(error.toString()).to.include("Title is too long");
    }

    // Test description too long
    try {
      await program.methods
        .addTodo("Title", "A".repeat(201))
        .accounts({
          signer: user.publicKey,
          todoAccount: todoPda,
          globalCounter: globalCounterPda,
        })
        .signers([user])
        .rpc();
      expect.fail("Should have failed with description too long");
    } catch (error) {
      expect(error.toString()).to.include("Description is too long");
    }
  });
});