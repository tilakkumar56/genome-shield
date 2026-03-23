import anchor from '@coral-xyz/anchor';
import { uploadCircuit } from '@arcium-hq/client';
import fs from 'fs';

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.ShadowVote;
const PROGRAM_ID = program.programId;

const circuit = fs.readFileSync('./build/cast_vote.arcis');
console.log('Circuit size:', circuit.length, 'bytes');

async function tryUpload(attempt) {
  try {
    console.log('Attempt', attempt, '...');
    const sig = await uploadCircuit(
      provider,
      'cast_vote',
      PROGRAM_ID,
      circuit,
      { skipPreflight: true, commitment: 'confirmed', maxRetries: 5 }
    );
    console.log('Upload complete:', sig);
    return true;
  } catch(e) {
    console.log('Attempt', attempt, 'error:', e.message?.slice(0, 200));
    return false;
  }
}

for (let i = 1; i <= 5; i++) {
  const ok = await tryUpload(i);
  if (ok) break;
  await new Promise(r => setTimeout(r, 3000));
}
