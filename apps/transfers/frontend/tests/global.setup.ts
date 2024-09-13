import { existsSync } from 'fs'
import { Readable } from 'stream'
import path from 'path'
import unzipper from 'unzipper'

async function globalSetup() {
  const folderName = `keplr-extension-manifest-v3-v${process.env.TEST_KEPLR_EXTENSION_VERSION}`
  const folderPath = path.join(__dirname, 'e2e', 'extensions', folderName)

  // Download and decompress Keplr extension if it does not exist yet
  if (!existsSync(folderPath)) {
    const downloadUrl = `https://github.com/chainapsis/keplr-wallet/releases/download/v${process.env.TEST_KEPLR_EXTENSION_VERSION}/${folderName}.zip`
    const resp = await fetch(downloadUrl)

    if (resp.ok && resp.body) {
      await Readable.fromWeb(resp.body as any)
        .pipe(unzipper.Extract({ path: folderPath }))
        .promise()
    }
  }
}

export default globalSetup
