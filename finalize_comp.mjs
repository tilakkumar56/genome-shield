import anchor from '@coral-xyz/anchor';
import { getMXEAccAddress, getCompDefAccAddress, getCompDefAccOffset, getArciumProgramId, getArciumProgram, getRawCircuitAccAddress } from '@arcium-hq/client';
import { PublicKey } from '@solana/web3.js';

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.ShadowVote;
const PROGRAM_ID = program.programId;

const compDefOffset = Buffer.from(getCompDefAccOffset('cast_vote')).readUInt32LE();
const compDefPDA = getCompDefAccAddress(PROGRAM_ID, compDefOffset);
const rawCircuit0 = getRawCircuitAccAddress(compDefPDA, 0);

const arcProg = getArciumProgram(provider);

try {
  const tx = await arcProg.methods.finalizeCompDef().accounts({
    compDefAcc: compDefPDA,
    compDefRaw: rawCircuit0,
    authority: provider.publicKey,
  }).rpc({ skipPreflight: true, commitment: 'confirmed' });
  console.log('Finalized:', tx);
} catch(e) {
  console.log('Finalize error:', e.message?.slice(0, 400));
  
  // Try alternate: just check state
  const arcProg2 = getArciumProgram(provider);
  try {
    const compDef = await arcProg2.account.computationDefinitionAccount.fetch(compDefPDA);
    console.log('CompDef status:', JSON.stringify(compDef.status));
    console.log('CompDef circuit source:', JSON.stringify(compDef.circuitSource));
  } catch(e2) {
    console.log('Fetch error:', e2.message?.slice(0, 200));
  }
}
