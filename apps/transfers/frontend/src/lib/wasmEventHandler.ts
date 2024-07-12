import { WebsocketClient } from '@cosmjs/tendermint-rpc'
import invariant from 'tiny-invariant'
import { Listener } from 'xstream'

// Connect and listen to blockchain events
export const wasmEventHandler = (
  query: string,
  listener: Partial<Listener<any>>,
): (() => void) => {
  invariant(
    process.env.NEXT_PUBLIC_CHAIN_RPC_URL,
    'NEXT_PUBLIC_CHAIN_RPC_URL must be defined',
  )
  // Create websocket connection to terdermint
  const websocketClient = new WebsocketClient(
    process.env.NEXT_PUBLIC_CHAIN_RPC_URL.replace('http', 'ws'),
  )

  // Listen to target query
  websocketClient
    .listen({
      jsonrpc: '2.0',
      method: 'subscribe',
      params: { query },
      // Just use some random UUID, we do not need to know which
      id: crypto.randomUUID(),
    })
    .subscribe(listener)

  // Callback method to call in case of cleaning
  return () => {
    websocketClient.disconnect()
  }
}
