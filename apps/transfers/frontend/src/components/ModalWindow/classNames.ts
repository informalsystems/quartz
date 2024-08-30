import { tw } from '@/lib/tw'
import { twMerge } from 'tailwind-merge'

export const classNames = {
  backdrop: ({ modalState = 'closed' }) =>
    twMerge(
      `
      fixed
      left-0
      top-0
      h-screen
      w-full
      transition-all
      z-10
      duration-500
    `,
      modalState === 'opening' || modalState === 'open'
        ? `
          opacity-100
          backdrop-blur-md
          pointer-events-auto
          bg-shadedBgColor/20
        `
        : `
          opacity-0
          backdrop-blur-none
          pointer-events-none
          bg-transparent
        `,
    ),

  container: ({ modalState = 'closed' }) =>
    twMerge(
      `
      fixed
      left-1/2
      top-1/2
      bg-appBgColor
      rounded-md
      overflow-hidden
      -translate-x-1/2
      z-10
      duration-500
      min-w-64
      shadow-md
      outline
      outline-1
      outline-black/5
    `,
      modalState === 'opening' || modalState === 'open'
        ? `
          opacity-100
          -translate-y-1/2
        `
        : `
          opacity-0
          pointer-events-none
          -translate-y-full
        `,
    ),

  header: tw`
    px-3
    py-1
    bg-accentColor
    text-white
    font-bold
  `,

  body: tw`
    p-3
  `,

  buttons: tw`
    flex
    flex-row-reverse
    gap-1
    p-3
  `,
}
