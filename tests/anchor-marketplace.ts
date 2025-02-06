import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorMarketplace } from "../target/types/anchor_marketplace";
import { createNft, findMasterEditionPda, findMetadataPda, mplTokenMetadata, verifySizedCollectionItem } from '@metaplex-foundation/mpl-token-metadata'
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { KeypairSigner, PublicKey, createSignerFromKeypair, generateSigner, keypairIdentity, percentAmount, publicKey } from '@metaplex-foundation/umi';
import { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";

describe("anchor-marketplace", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.AnchorMarketplace as Program<AnchorMarketplace>;
  const connection = provider.connection;
  const umi = createUmi(provider.connection);
  const payer = provider.wallet as NodeWallet;

  let nftMint: KeypairSigner = generateSigner(umi);
  let collectionMint: KeypairSigner = generateSigner(umi);

  const creatorWallet = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(payer.payer.secretKey));
  const creator = createSignerFromKeypair(umi, creatorWallet);
  umi.use(keypairIdentity(creator));
  umi.use(mplTokenMetadata());

  let makerAta: anchor.web3.PublicKey;
  let takerAta: anchor.web3.PublicKey;
  let vault: anchor.web3.PublicKey;

  const maker = Keypair.generate();
  const taker = Keypair.generate();

  const name = "user123"; 
  const price = new anchor.BN(1);

  const marketplace = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("marketplace"), Buffer.from(name)], program.programId)[0];
  const rewardsMint = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("rewards"), marketplace.toBuffer()], program.programId)[0];
  const treasury = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("treasury"), marketplace.toBuffer()], program.programId)[0];
  const listing = anchor.web3.PublicKey.findProgramAddressSync([marketplace.toBuffer(), new anchor.web3.PublicKey(nftMint.publicKey as PublicKey).toBuffer()], program.programId)[0];

  before(async () => {
    // Airdrop SOL to maker and taker
    const makerAirdrop = await connection.requestAirdrop(maker.publicKey, 7 * LAMPORTS_PER_SOL);
    const takerAirdrop = await connection.requestAirdrop(taker.publicKey, 7 * LAMPORTS_PER_SOL);
    const latestBlockhash = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature: makerAirdrop, ...latestBlockhash });
    await connection.confirmTransaction({ signature: takerAirdrop, ...latestBlockhash });
    await sleep(2000);

    // Mint Collection NFT
    await createNft(umi, {
      mint: collectionMint,
      name: "GM",
      symbol: "GM",
      uri: "https://arweave.net/123",
      sellerFeeBasisPoints: percentAmount(5.5),
      collectionDetails: { __kind: 'V1', size: 10 }
    }).sendAndConfirm(umi);
    console.log(`Created Collection NFT: ${collectionMint.publicKey.toString()}`);

    // Mint NFT into maker's ATA
    await createNft(umi, {
      mint: nftMint,
      name: "GM",
      symbol: "GM",
      uri: "https://arweave.net/123",
      sellerFeeBasisPoints: percentAmount(5.5),
      collection: { verified: false, key: collectionMint.publicKey },
      tokenOwner: publicKey(maker.publicKey) // Corrected to use maker's public key
    }).sendAndConfirm(umi);
    console.log(`Created NFT: ${nftMint.publicKey.toString()}`);

    // Verify Collection
    const collectionMetadata = findMetadataPda(umi, { mint: collectionMint.publicKey });
    const collectionMasterEdition = findMasterEditionPda(umi, { mint: collectionMint.publicKey });
    const nftMetadata = findMetadataPda(umi, { mint: nftMint.publicKey });
    await verifySizedCollectionItem(umi, {
      metadata: nftMetadata,
      collectionAuthority: creator,
      collectionMint: collectionMint.publicKey,
      collection: collectionMetadata,
      collectionMasterEditionAccount: collectionMasterEdition,
    }).sendAndConfirm(umi);
    console.log("Collection NFT Verified!");

    // Get or create ATAs
    makerAta = (await getOrCreateAssociatedTokenAccount(
      connection,
      maker,
      new anchor.web3.PublicKey(nftMint.publicKey),
      maker.publicKey
    )).address;

    takerAta = (await getOrCreateAssociatedTokenAccount(
      connection,
      taker,
      new anchor.web3.PublicKey(nftMint.publicKey),
      taker.publicKey
    )).address;

    vault = await anchor.utils.token.associatedAddress({
      mint: new anchor.web3.PublicKey(nftMint.publicKey),
      owner: listing,
    });
  });

  it("Initialize Marketplace!", async () => {
    const tx = await program.methods.initialize(name, 1)
      .accountsPartial({
        admin: provider.wallet.publicKey,
        marketplace,
        rewardsMint,
        treasury,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Marketplace Initialized. Tx:", tx);
  });

  
  it("Listing!", async () => {
    
    const nftMetadata = findMetadataPda(umi, {mint: nftMint.publicKey});
    const nftEdition = findMasterEditionPda(umi, {mint: nftMint.publicKey});
  
    // Add your test here.
    const tx = await program.methods.list(price)
    .accountsPartial({
      maker: maker.publicKey,
      marketplace,
      makerMint: nftMint.publicKey,
      collectionMint: collectionMint.publicKey,
      makerAta,
      metadata: new anchor.web3.PublicKey(nftMetadata[0]),
      vault,
      masterEdition: new anchor.web3.PublicKey(nftEdition[0]),
      listing,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([maker])
    .rpc();
    console.log("\nListing Initialized!");
    console.log("Your transaction signature", tx);
  });
  
  it("Delisting!", async () => {
  
    // Add your test here.
    const tx = await program.methods.delist()
    .accountsPartial({
      maker: maker.publicKey,
      marketplace,
      makerMint: nftMint.publicKey,
      makerAta,
      listing,
      vault,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([maker])
    .rpc();
    console.log("\nDelisting Initialized!");
    console.log("Your transaction signature", tx);
  });
  
  // it("Purchase Initialized!", async () => {
    
  //   // Add your test here.
  //   const tx = await program.methods.purchase()
  //   .accountsPartial({
  //     taker: taker.publicKey,
  //     maker: maker.publicKey,
  //     makerMint: nftMint.publicKey,
  //     marketplace,
  //     takerAta,
  //     vault,
  //     rewardsMint,
  //     listing,
  //     treasury,
  //     systemProgram: anchor.web3.SystemProgram.programId,
  //     tokenProgram: TOKEN_PROGRAM_ID,
  //   })
  //   .signers([taker])
  //   .rpc();
  //   console.log("\nPurchase Initialized!");
  //   console.log("Your transaction signature", tx);
  // });
  
});

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

