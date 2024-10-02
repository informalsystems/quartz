import invariant from 'tiny-invariant'

export async function register() {
  // Ensure all required env vars are set before starting the server
  invariant(
    process.env.NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY,
    'NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY environment variable must be defined',
  )
  invariant(
    process.env.NEXT_PUBLIC_TARGET_CHAIN,
    'NEXT_PUBLIC_TARGET_CHAIN environment variable must be defined',
  )
  invariant(
    process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS,
    'NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS environment variable must be defined',
  )
}
