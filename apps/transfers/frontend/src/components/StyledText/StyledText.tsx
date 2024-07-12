import { ComponentProps, ElementType } from 'react'
import { twMerge } from 'tailwind-merge'
import { classNames } from './classNames'

type StyledTextVariant = keyof typeof classNames

type StyledTextProps<T extends ElementType = 'span'> = Omit<
  ComponentProps<T>,
  'variant'
> & {
  as?: T
  variant?: StyledTextVariant
}

export function StyledText<T extends ElementType = 'span'>({
  as,
  className,
  variant,
  ...otherProps
}: StyledTextProps<T>) {
  const Component = as || 'span'

  return (
    <Component
      className={twMerge(variant && classNames[variant], className)}
      {...otherProps}
    />
  )
}
