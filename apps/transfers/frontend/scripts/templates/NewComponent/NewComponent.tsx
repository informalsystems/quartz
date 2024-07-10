import { ComponentProps } from 'react'
import { twMerge } from 'tailwind-merge'
import { classNames } from './classNames'

interface NewComponentProps extends ComponentProps<'div'> {}

export function NewComponent({
  children,
  className,
  ...otherProps
}: NewComponentProps) {
  return (
    <div
      className={twMerge(classNames.container, className)}
      {...otherProps}
    >
      {children}
    </div>
  )
}
