import { ComponentProps, ElementType } from 'react'
import { twMerge } from 'tailwind-merge'
import { classNames } from './classNames'

type StyledBoxVariant = keyof typeof classNames

type StyledBoxProps<T extends ElementType = 'div'> = Omit<
  ComponentProps<T>,
  'variant'
> & {
  as?: T
  variant?: StyledBoxVariant
}

export function StyledBox<T extends ElementType = 'div'>({
  as,
  className,
  variant,
  ...otherProps
}: StyledBoxProps<T>) {
  const Component = as || 'div'

  return (
    <Component
      className={twMerge(variant && classNames[variant], className)}
      {...otherProps}
    />
  )
}
