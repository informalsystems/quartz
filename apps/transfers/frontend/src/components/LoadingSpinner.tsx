import { Icon } from '@/components/Icon'
import { ComponentProps } from 'react'
import { twMerge } from 'tailwind-merge'

interface LoadingSpinnerProps extends Omit<ComponentProps<'div'>, 'children'> {
  isLoading: boolean
}

export function LoadingSpinner({
  className,
  isLoading,
  ...otherProps
}: LoadingSpinnerProps) {
  return (
    <div
      className={twMerge(
        `
          absolute
          bottom-0
          left-0
          right-0
          top-0
          z-20
          flex
          items-center
          justify-center
          bg-gray-500/25
          transition-all
        `,
        isLoading
          ? `
              opacity-100
              `
          : `
              opacity-0
              pointer-events-none
            `,
        className,
      )}
      {...otherProps}
    >
      <div className="animate-spin">
        <Icon name="spinner" />
      </div>
    </div>
  )
}
