import { useGlobalState } from '@/state/useGlobalState'
import { LoadingSpinner } from '@/components/LoadingSpinner'

export const LoadingWrapper = () => {
  const loading = useGlobalState((state) => state.loading)

  return <LoadingSpinner isLoading={loading} />
}
