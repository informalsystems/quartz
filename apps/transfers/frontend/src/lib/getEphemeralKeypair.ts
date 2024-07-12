import {
  Bip39,
  EnglishMnemonic,
  Random,
  Secp256k1,
  Secp256k1Keypair,
} from '@cosmjs/crypto'

const storageKey = 'ephemeral-mnemonic'

// Retrieve from localstorage a mnemonic or create it if none exists to be used
// as input to generate an ephemeral key pair to encryp/decrypt messages for the user
export async function getEphemeralKeypair(): Promise<Secp256k1Keypair> {
  const storedMnemonic = localStorage.getItem(storageKey)
  const mnemonic =
    storedMnemonic ?? Bip39.encode(Random.getBytes(32)).toString()

  if (!storedMnemonic) {
    localStorage.setItem(storageKey, mnemonic)
  }

  return Secp256k1.makeKeypair(Bip39.decode(new EnglishMnemonic(mnemonic)))
}
