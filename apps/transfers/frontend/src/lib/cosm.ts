import { toUtf8 } from '@cosmjs/encoding'
import { Registry } from '@cosmjs/proto-signing'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { coins } from '@cosmjs/stargate'
import { MsgExecuteContract } from 'cosmjs-types/cosmwasm/wasm/v1/tx'
import invariant from 'tiny-invariant'

import { wallet } from './wallet'

const typeUrl = '/cosmwasm.wasm.v1.MsgExecuteContract'
const registry = new Registry([[typeUrl, MsgExecuteContract]])

// Cosm variables declaration. They will be set upon initialization.
let signingCosmClient: SigningCosmWasmClient;



// Setup the CosmWasm client.
const init = async () => {
  invariant(
    process.env.NEXT_PUBLIC_CHAIN_RPC_URL,
    'NEXT_PUBLIC_CHAIN_RPC_URL must be defined',
  )


  // Initialize Cosm client.
  signingCosmClient = await SigningCosmWasmClient.connectWithSigner(
    process.env.NEXT_PUBLIC_CHAIN_RPC_URL,
    wallet.getSigner(),
    { registry },
 )

}
// Transfer contract execution message
const executeTransferContract = ({
  messageBuilder,
  fundsAmount,
}: {
  messageBuilder: Function
  fundsAmount?: string
}) => {
  invariant(
    process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS,
    'NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS must be defined',
  )

  const sender = wallet.getAccount().address
  // Prepare execution message to send
  const executeTransferContractMsgs = [
    {
      typeUrl,
      value: MsgExecuteContract.fromPartial({
        sender,
        contract: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS,
        msg: toUtf8(JSON.stringify(messageBuilder())),
        ...(fundsAmount && {
          funds: [{ denom: 'untrn', amount: fundsAmount }],
        }),
      }),
    },
  ]

  // Send message
  return signingCosmClient.signAndBroadcast(
    sender,
    executeTransferContractMsgs,
    {
      amount: coins(1, 'untrn'),
      gas: '400000',
    },
  )
}
// Transfer contract query message
const queryTransferContract = ({
  messageBuilder,
}: {
  messageBuilder: Function
}) => {
  invariant(
    process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS,
    'NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS must be defined',
  )

  // Send message
  return signingCosmClient.queryContractSmart(
    process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS,
    messageBuilder(),
  )
}

// Define the Cosm wrapper interface to be used
export const cosm = {
  executeTransferContract,
  init,
  queryTransferContract,
}
