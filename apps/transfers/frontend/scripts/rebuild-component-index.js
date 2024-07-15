const fs = require('fs')

const componentsDirectory = `${__dirname}/../src/components`
const indexFile = `${componentsDirectory}/index.ts`

// Get directories and files in the components directory
const filesAndDirectories = fs.readdirSync(componentsDirectory)

const [files, directories] = filesAndDirectories.reduce(
  (acc, fileOrDirectory) => {
    if (
      fs.statSync(`${componentsDirectory}/${fileOrDirectory}`).isDirectory()
    ) {
      acc[1].push(fileOrDirectory)
    } else {
      acc[0].push(fileOrDirectory)
    }

    return acc
  },
  [[], []],
)

// Write them
fs.writeFileSync(
  indexFile,
  filesAndDirectories
    .reduce((acc, fileOrDirectory) => {
      if (['.DS_Store', 'index.ts'].includes(fileOrDirectory)) {
        return [...acc]
      }

      const componentName = fileOrDirectory.replace('.tsx', '')

      return [...acc, `export { ${componentName} } from './${componentName}'`]
    }, [])
    .join('\n'),
  {
    flag: 'w',
  },
)

console.log(`Rebuilt component index file at ${indexFile}`)
