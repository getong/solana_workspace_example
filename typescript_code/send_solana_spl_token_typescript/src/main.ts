import {
    Connection,
    PublicKey,
    Keypair,
    clusterApiUrl,
    LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    getMint,
    createMint,
    getOrCreateAssociatedTokenAccount,
    mintTo,
    transfer
} from "@solana/spl-token";
import * as fs from 'fs';
import bs58 from 'bs58';

(async (): Promise<void> => {
    // Connect to cluster
    const connection: Connection = new Connection(clusterApiUrl('devnet'), 'confirmed');

    // Option 1: Generate a new keypair for testing
    // const fromWallet: Keypair = Keypair.generate();

    // Option 2: Load from a file (safer approach for real keys)
    // const secretKeyString: string = fs.readFileSync('/path/to/keypair.json', 'utf-8');
    // const secretKey: Uint8Array = Uint8Array.from(JSON.parse(secretKeyString));
    // const fromWallet: Keypair = Keypair.fromSecretKey(secretKey);

    // Option 3: Load from base58 encoded string
    const secretKeyBase58: string = "YOUR_BASE58_SECRET_KEY"; // Replace with your actual base58 secret key
    if (
        !secretKeyBase58 ||
        secretKeyBase58 === "YOUR_BASE58_SECRET_KEY"
    ) {
        throw new Error(
            "Please replace 'YOUR_BASE58_SECRET_KEY' with your actual base58-encoded secret key."
        );
    }
    const decodedKey: Uint8Array = bs58.decode(secretKeyBase58);
    const fromWallet: Keypair = Keypair.fromSecretKey(
        Uint8Array.from(decodedKey)
    );

    // Generate a new wallet to receive newly minted token
    const walletTo: string = "YOUR WALLET";
    const destPublicKey: PublicKey = new PublicKey(walletTo);
    const destMint: PublicKey = new PublicKey("YOUR TOKEN ADDRESS");

    const tokenM: PublicKey = new PublicKey("YOUR TOKEN ADDRESS");

    // Get the token account of the fromWallet address, and if it does not exist, create it
    const fromTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        fromWallet,
        tokenM,
        fromWallet.publicKey
    );

    // Get the token account of the toWallet address, and if it does not exist, create it
    const toTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        fromWallet,
        tokenM,
        destPublicKey
    );

    // Mint 1 new token to the "fromTokenAccount" account we just created
    let signature = await mintTo(
        connection,
        fromWallet,
        destMint,
        fromTokenAccount.address,
        fromWallet.publicKey,
        3 * LAMPORTS_PER_SOL
    );
    console.log('mint tx:', signature);

    // Transfer the new token to the "toTokenAccount" we just created
    signature = await transfer(
        connection,
        fromWallet,
        fromTokenAccount.address,
        toTokenAccount.address,
        fromWallet.publicKey,
        10 * LAMPORTS_PER_SOL
    );
})();