import { create } from 'zustand'

interface GlobalState {
  loading: boolean
  setLoading: (loading: boolean) => void
}

export const useGlobalState = create<GlobalState>((set) => ({
  loading: true,
  setLoading: (loading: boolean) => set({ loading }),
}))
