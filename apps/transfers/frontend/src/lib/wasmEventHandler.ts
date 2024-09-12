import { WebsocketClient } from '@cosmjs/tendermint-rpc'
import { Listener } from 'xstream'

import chain from '@/config/chain'

// Connect and listen to blockchain events
export const wasmEventHandler = (
  query: string,
  listener: Partial<Listener<any>>,
): (() => void) => {
  // Create websocket connection to terdermint
  const websocketClient = new WebsocketClient(chain.rpc.replace('http', 'ws'))

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
