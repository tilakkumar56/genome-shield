import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import {
  x25519, RescueCipher, getMXEPublicKey, getMXEAccAddress,
  getCompDefAccAddress, getClusterAccAddress, getComputationAccAddress,
  getMempoolAccAddress, getExecutingPoolAccAddress, getFeePoolAccAddress,
  getClockAccAddress, getCompDefAccOffset, getArciumEnv,
  awaitComputationFinalization, deserializeLE, uploadCircuit, getCircuitState,
} from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

function readKpJson(path: string): anchor.web3.Keypair {
  const raw = JSON.parse(fs.readFileSync(path, "utf-8"));
  return anchor.web3.Keypair.fromSecretKey(new Uint8Array(raw));
}

describe("shadow_vote", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.ShadowVote;
  const provider = anchor.getProvider();
  const arciumEnv = getArciumEnv();

  it("Init comp def and cast vote", async () => {
    const owner = readKpJson(`${os.homedir()}/.config/solana/id.json`);

    // Check circuit state
    const state = await getCircuitState(provider as anchor.AnchorProvider, program.programId, "cast_vote");
    console.log("Circuit state:", state);

    if (state !== "Finalized") {
      // Upload circuit
      const circuit = fs.readFileSync("./build/cast_vote.arcis");
      console.log("Uploading circuit:", circuit.length, "bytes");
      await uploadCircuit(
        provider as anchor.AnchorProvider,
        "cast_vote",
        program.programId,
        circuit,
        { skipPreflight: true, commitment: "confirmed" }
      );
      console.log("Circuit uploaded");
    }

    // Now test the actual MPC flow
    const mxePubKey = await getMXEPublicKey(provider as anchor.AnchorProvider, program.programId);
    console.log("MXE pubkey:", mxePubKey);

    const privKey = x25519.utils.randomPrivateKey();
    const pubKey = x25519.getPublicKey(privKey);
    const sharedSecret = x25519.getSharedSecret(privKey, mxePubKey);
    const cipher = new RescueCipher(sharedSecret);
    const nonce = randomBytes(16);

    const ctOptionIdx = cipher.encrypt([BigInt(1)], nonce);
    const ctWeight = cipher.encrypt([BigInt(1)], nonce);
    const ctNumOptions = cipher.encrypt([BigInt(4)], nonce);

    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const compDefOffset = Buffer.from(getCompDefAccOffset("cast_vote")).readUInt32LE();

    console.log("Queuing vote computation...");
    const queueTx = await program.methods.castVote(
      computationOffset,
      Array.from(ctOptionIdx[0]),
      Array.from(ctWeight[0]),
      Array.from(ctNumOptions[0]),
      Array.from(pubKey),
      new anchor.BN(deserializeLE(nonce).toString()),
    ).accountsPartial({
      computationAccount: getComputationAccAddress(arciumEnv.arciumClusterOffset, computationOffset),
      clusterAccount: getClusterAccAddress(arciumEnv.arciumClusterOffset),
      mxeAccount: getMXEAccAddress(program.programId),
      mempoolAccount: getMempoolAccAddress(arciumEnv.arciumClusterOffset),
      executingPool: getExecutingPoolAccAddress(arciumEnv.arciumClusterOffset),
      compDefAccount: getCompDefAccAddress(program.programId, compDefOffset),
      poolAccount: getFeePoolAccAddress(),
      clockAccount: getClockAccAddress(),
    }).rpc({ skipPreflight: true, commitment: "confirmed" });

    console.log("Queue tx:", queueTx);

    const finalizeTx = await awaitComputationFinalization(
      provider as anchor.AnchorProvider,
      computationOffset,
      program.programId,
      "confirmed"
    );
    console.log("Finalize tx:", finalizeTx);
    console.log("MPC vote computation completed successfully!");
  });
});
