import { tw } from '@/lib/tw'

export const classNames = {
  container: tw`
    container
    mx-auto
    max-sm:px-6
    md:max-w-screen-sm
    lg:max-w-screen-md
    xl:max-w-screen-lg
    2xl:max-w-screen-xl
  `,

  checkbox: tw`
    text-bgColor
    size-7
    rounded-md
    border-2
    border-borderColor
    !outline-accentColor
    !ring-accentColor
    accent-accentColor
    checked:text-accentColor
  `,

  input: tw`
    w-full
    rounded-md
    border
    border-borderColor
    bg-transparent
    px-3
    py-2
    !outline-accentColor
    !ring-accentColor
    focus:border-accentColor
    [appearance:textfield]
  `,
}
