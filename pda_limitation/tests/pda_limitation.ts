import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PdaLimitation } from "../target/types/pda_limitation";
import { expect } from "chai";

describe("pda_limitation", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.pdaLimitation as Program<PdaLimitation>;
  const provider = anchor.getProvider();
  const user = (provider.wallet as anchor.Wallet).payer;

  let todoListPda: anchor.web3.PublicKey;
  let todoListBump: number;

  before(async () => {
    [todoListPda, todoListBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("todo_list"), user.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initialize todo list", async () => {
    const tx = await program.methods
      .initializeTodoList()
      .accounts({
        todoList: todoListPda,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Initialize todo list transaction signature", tx);

    const todoListAccount = await program.account.todoList.fetch(todoListPda);
    expect(todoListAccount.owner.toString()).to.equal(user.publicKey.toString());
    expect(todoListAccount.todoCount.toNumber()).to.equal(0);
  });

  it("Create a todo item", async () => {
    const title = "Test Todo";
    const description = "This is a test todo item";

    const [todoItemPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("todo_item"),
        user.publicKey.toBuffer(),
        new anchor.BN(0).toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    const tx = await program.methods
      .createTodo(title, description)
      .accounts({
        todoItem: todoItemPda,
        todoList: todoListPda,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Create todo transaction signature", tx);

    const todoItemAccount = await program.account.todoItem.fetch(todoItemPda);
    expect(todoItemAccount.title).to.equal(title);
    expect(todoItemAccount.description).to.equal(description);
    expect(todoItemAccount.completed).to.be.false;
    expect(todoItemAccount.id.toNumber()).to.equal(0);

    const todoListAccount = await program.account.todoList.fetch(todoListPda);
    expect(todoListAccount.todoCount.toNumber()).to.equal(1);
  });

  it("Update a todo item", async () => {
    const newTitle = "Updated Todo";
    const newDescription = "This is an updated todo item";

    const [todoItemPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("todo_item"),
        user.publicKey.toBuffer(),
        new anchor.BN(0).toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    const tx = await program.methods
      .updateTodo(newTitle, newDescription, true)
      .accounts({
        todoItem: todoItemPda,
        user: user.publicKey,
        owner: user.publicKey,
      })
      .signers([user])
      .rpc();

    console.log("Update todo transaction signature", tx);

    const todoItemAccount = await program.account.todoItem.fetch(todoItemPda);
    expect(todoItemAccount.title).to.equal(newTitle);
    expect(todoItemAccount.description).to.equal(newDescription);
    expect(todoItemAccount.completed).to.be.true;
  });

  it("Delete a todo item", async () => {
    const [todoItemPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("todo_item"),
        user.publicKey.toBuffer(),
        new anchor.BN(0).toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    const tx = await program.methods
      .deleteTodo()
      .accounts({
        todoItem: todoItemPda,
        user: user.publicKey,
        owner: user.publicKey,
      })
      .signers([user])
      .rpc();

    console.log("Delete todo transaction signature", tx);

    // Verify the account is closed
    try {
      await program.account.todoItem.fetch(todoItemPda);
      expect.fail("Todo item should have been deleted");
    } catch (error) {
      expect(error.message).to.include("Account does not exist");
    }
  });

  it("Test large description (near 10MB limit)", async () => {
    // Create a large description close to the 10MB limit
    const largeDescription = "A".repeat(10 * 1024 * 1024 - 400); // ~10MB minus overhead
    const title = "Large Todo";

    const [todoItemPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("todo_item"),
        user.publicKey.toBuffer(),
        new anchor.BN(1).toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    const tx = await program.methods
      .createTodo(title, largeDescription)
      .accounts({
        todoItem: todoItemPda,
        todoList: todoListPda,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Create large todo transaction signature", tx);

    const todoItemAccount = await program.account.todoItem.fetch(todoItemPda);
    expect(todoItemAccount.title).to.equal(title);
    expect(todoItemAccount.description).to.equal(largeDescription);
    expect(todoItemAccount.completed).to.be.false;
  });
});
