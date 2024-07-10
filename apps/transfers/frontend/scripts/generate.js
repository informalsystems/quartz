const fs = require('fs')
const readline = require('node:readline')

const componentsDirectory = `${__dirname}/../src/components`

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
})

rl.question(
  `What do you want to name your new component? `,
  (newComponentName) => {
    newComponentName = newComponentName || 'NewComponent'

    rl.question(
      `What tag name do you want to use for your new component? (default: div) `,
      (tagName) => {
        tagName = tagName || 'div'

        rl.question(
          `Is this a complex component? (Y/N, default: N)`,
          (isComplex) => {
            isComplex = isComplex === 'Y' ? true : false

            const replacementsMap = {
              'NewComponent': newComponentName,
              'div': tagName,
              '//[^\n]+': '',
            }

            if (isComplex) {
              const componentDirectory = `${componentsDirectory}/${newComponentName}`

              // Make sure components directory exists
              fs.mkdirSync(componentsDirectory, { recursive: true })

              // Create the new component directory and copy the template files
              fs.cpSync(
                `${__dirname}/templates/NewComponent`,
                componentDirectory,
                {
                  recursive: true,
                },
              )

              const template = fs.readFileSync(
                `${__dirname}/templates/NewComponent/NewComponent.tsx`,
                'utf8',
              )

              const mergedTemplate = Object.entries(replacementsMap).reduce(
                (acc, [key, value]) => {
                  return acc.replace(new RegExp(`${key}`, 'isg'), value)
                },
                template,
              )

              const newComponentFile = `${componentDirectory}/NewComponent.tsx`

              fs.writeFileSync(newComponentFile, mergedTemplate)

              fs.writeFileSync(
                `${componentDirectory}/index.ts`,
                `export { ${newComponentName} } from './${newComponentName}'\n`,
              )

              fs.renameSync(
                newComponentFile,
                `${componentDirectory}/${newComponentName}.tsx`,
              )
            } else {
              const newComponentFile = `${componentsDirectory}/${newComponentName}.tsx`

              fs.cpSync(
                `${__dirname}/templates/NewComponent.tsx`,
                newComponentFile,
              )

              const template = fs.readFileSync(
                `${__dirname}/templates/NewComponent.tsx`,
                'utf8',
              )

              const mergedTemplate = Object.entries(replacementsMap).reduce(
                (acc, [key, value]) => {
                  return acc.replace(new RegExp(`${key}`, 'isg'), value)
                },
                template,
              )

              fs.writeFileSync(newComponentFile, mergedTemplate)
            }

            console.log(
              `Component ${newComponentName} created at ${componentsDirectory}`,
            )

            rl.close()
          },
        )
      },
    )
  },
)
