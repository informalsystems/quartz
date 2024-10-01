import { fromBech32 } from '@cosmjs/encoding'

export const isValidAddress = (address: string): boolean => {
  try {
    fromBech32(address)

    return true
  } catch {
    return false
  }
}
