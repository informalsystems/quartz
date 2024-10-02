import { closeSnackbar, enqueueSnackbar, VariantType } from 'notistack'

import { Icon } from '@/components/Icon'
import { StyledText } from '@/components/StyledText'

type Notification = {
  message: string
  dismissible?: boolean
  variant?: VariantType
}

export const showNotification = ({
  message,
  dismissible = false,
  variant = 'info',
}: Notification) =>
  enqueueSnackbar(message, {
    variant,
    ...(dismissible && {
      persist: true,
      action: (snackbarId) => (
        <StyledText
          as="button"
          variant="button.tool"
          onClick={() => closeSnackbar(snackbarId)}
        >
          <Icon
            className="text-white"
            name="xmark"
          />
        </StyledText>
      ),
    }),
  })

export const showSuccess = (message: string) =>
  showNotification({ message, variant: 'success' })

export const showError = (message: string) =>
  showNotification({ message, dismissible: true, variant: 'error' })
