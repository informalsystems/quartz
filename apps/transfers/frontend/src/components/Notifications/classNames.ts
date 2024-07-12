import { tw } from '@/lib/tw'
import { twMerge } from 'tailwind-merge'

export const classNames = {
  backdrop: ({ hasMessages = false, success = false }) =>
    twMerge(
      `
        fixed
        bottom-0
        right-0
        h-[20vh]
        w-[50vw]
        bg-gradient-to-tl
        via-transparent
        to-transparent
        opacity-0
        transition-opacity
        duration-500
        z-10
      `,
      success ? 'from-green-400/50' : 'from-red-400/50',
      hasMessages &&
        `
          opacity-100
        `,
    ),

  container: tw`
    fixed
    bottom-0
    right-0
    flex
    flex-col-reverse
    gap-2
    p-6
    z-20
  `,

  notificationContainer: ({ state = 'entering' }) =>
    twMerge(
      `
        w-[20rem]
        overflow-hidden
        transition-all
      `,
      state === 'entering' &&
        `
          animate-animateInX
        `,
      state === 'entered' &&
        `
          opacity-100
          shadow-md
        `,
      state === 'exiting' &&
        `
          animate-animateOutX
        `,
      state === 'exited' &&
        `
          hidden
        `,
    ),

  notificationSurface: ({ success = false }) =>
    twMerge(
      `
        grid
        w-full
        grid-cols-[auto,1fr,auto]
        gap-3
        rounded-md
        border-2
        px-3
        py-3
        text-sm
        text-white
        transition-all
      `,
      success
        ? `
            border-green-400
            bg-green-500
          `
        : `
            border-red-400
            bg-red-500
          `,
    ),
}
