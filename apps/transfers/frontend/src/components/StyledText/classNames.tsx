import { tw } from '@/lib/tw'
import { twJoin, twMerge } from 'tailwind-merge'

const buttonClassNames = tw`
  inline-flex
  w-min
  cursor-pointer
  items-center
  justify-center
  gap-2
  font-bold
  transition-all
  hover:scale-105
  disabled:pointer-events-none
  disabled:opacity-40
`

const headingClassNames = tw`
  font-medium
  [&_span]:font-display
  [&_span]:font-medium
  [&_span]:italic
  [&_span]:text-accentColor
`

export const classNames = {
  'button.primary': twMerge(
    buttonClassNames,
    `whitespace-nowrap rounded-md bg-accentColor px-3 py-1 text-white`,
  ),

  'button.secondary': twMerge(
    buttonClassNames,
    `relative z-10 whitespace-nowrap rounded-md border bg-gray-400 px-3 py-1 text-white backdrop-blur-sm`,
  ),

  'button.icon': twMerge(
    buttonClassNames,
    `size-10 rounded-full bg-shadedBgColor hover:bg-accentColor hover:text-appBgColor`,
  ),

  'button.tool': tw`
    inline-flex
    items-center
    justify-center
    gap-2
    rounded-sm
    px-1
    py-0.5
    text-sm
    text-textColor
    hover:bg-textColor/15
  `,

  'footnote': tw`
    text-sm
    text-fadedTextColor
  `,

  'h1': twJoin(headingClassNames, `text-5xl`),

  'h2': twJoin(headingClassNames, `text-4xl`),

  'h3': twJoin(headingClassNames, `text-3xl`),

  'h4': twJoin(headingClassNames, `text-lg`),

  'label': tw`
    text-sm
    text-fadedTextColor
  `,

  'link': twMerge(
    buttonClassNames,
    `text-accentColor underline underline-offset-4 transition-all hover:underline-offset-4`,
  ),

  'link.subtle': twMerge(
    buttonClassNames,
    `text-inherit underline-offset-4 transition-all hover:text-accentColor hover:underline hover:underline-offset-4`,
  ),

  'logo': tw`
    inline-flex
    items-center
    justify-center
    gap-[0.35em]
    font-semibold
    uppercase
    tracking-[0.2em]
    [&_span]:inline-block
    [&_span]:border-l
    [&_span]:pl-[0.5em]
    [&_span]:font-light
    [&_span]:tracking-[0]
  `,
}
