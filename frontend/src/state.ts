import { computed, signal } from '@preact/signals-react'

export type Session = {
  userId: number
  username: string
  token: string
}

export const apiBaseSignal = signal(import.meta.env.VITE_API_BASE ?? 'http://localhost:8080')
export const sessionSignal = signal<Session | null>(null)
export const authHeaderSignal = computed(() =>
  sessionSignal.value ? { Authorization: `Bearer ${sessionSignal.value.token}` } : {},
)
