// Transfer contract operation message formats
// This is the whole definition of how the Tranfers contract expects to receive messages
export const contractMessageBuilders: {
  deposit: () => {}
  getBalance: (address: string) => {
    get_balance: { address: string }
  }
  requestBalance: (pubkey: Uint8Array) => {}
  transfer: (ciphertext: string) => {
    transfer_request: {
      ciphertext: string
      digest: string
    }
  }
  withdraw: () => {}
} = {
  deposit: () => 'deposit',
  getBalance: (address: string) => ({
    get_balance: { address },
  }),
  requestBalance: (pubkey: Uint8Array) => ({
    query_request: { emphemeral_pubkey: Buffer.from(pubkey).toString('hex') },
  }),
  transfer: (ciphertext: string) => ({
    transfer_request: { ciphertext, digest: '' },
  }),
  withdraw: () => 'withdraw',
}
