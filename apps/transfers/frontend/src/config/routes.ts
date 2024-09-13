type Url = 'landing' | 'seed' | 'dashboard'

export const routes: Record<Url, string> = {
  landing: '/',
  seed: '/set-seed',
  dashboard: '/dashboard',
}
