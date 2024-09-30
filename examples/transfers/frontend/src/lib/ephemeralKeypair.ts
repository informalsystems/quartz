import {
  Bip39,
  EnglishMnemonic,
  Random,
  Secp256k1,
  Secp256k1Keypair,
} from '@cosmjs/crypto'
import { decrypt as _decrypt } from 'eciesjs'
import { isEmpty } from 'lodash'

// Generate a random Mnemonic
export function generateMnemonic() {
  return Bip39.encode(Random.getBytes(32)).toString()
}
// Retrieve the Mnemonic from storage
export function getMnemonic() {
  return localStorage.getItem('ephemeral-mnemonic')!
}
// Save Mnemonic into storage
export function saveMnemonic(mnemonic: string) {
  localStorage.setItem('ephemeral-mnemonic', mnemonic)
}
// Clear stored mnemonic
export function clearMnemonic() {
  localStorage.removeItem('ephemeral-mnemonic')
}
// Generate an ephemeral key pair to encryp/decrypt messages for the user from stored mnemonic
export async function getEphemeralKeypair(): Promise<Secp256k1Keypair> {
  let privkeyFromMnemonic = Bip39.decode(new EnglishMnemonic(getMnemonic()))

  // If mnemonic is not formed by 24 words, lets expand it
  if (privkeyFromMnemonic.length < 32) {
    const newPrivKey = new Uint8Array(32)

    newPrivKey.set(privkeyFromMnemonic)

    privkeyFromMnemonic = newPrivKey
  }

  return Secp256k1.makeKeypair(privkeyFromMnemonic)
}
// Decrypt data using the ephemeral private key
export async function decrypt(data?: string): Promise<string> {
  if (isEmpty(data)) {
    return ''
  }

  return _decrypt(
    (await getEphemeralKeypair()).privkey,
    Buffer.from(data!, 'hex'),
  ).toString()
}
