import useSWR from 'swr'

export function useLogin (props = {}) {
  const { data, error } = useSWR('/login')
  if (error || !data || !data.ok) return { ok: false, user: null }
  return {
    ok: true,
    user: data.user
  }
}
