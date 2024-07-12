import { ComponentProps } from 'react'
import { twMerge } from 'tailwind-merge'

interface NewComponentProps extends ComponentProps<'div'> {}

export function NewComponent({
  children,
  className,
  ...otherProps
}: NewComponentProps) {
  return (
    <div
      className={twMerge(
        `
          
        `,
        className,
      )}
      {...otherProps}
    >
      {children}
    </div>
  )
}
