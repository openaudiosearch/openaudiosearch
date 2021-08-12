import useSWR from 'swr'

export function useLogin (props = {}) {
  const { data, error } = useSWR('/login')
  if (error || !data || !data.ok) return { ok: false, user: null }
  return {
    ok: true,
    user: data.user
  }
}

export function useIsAdmin () {
  const { user } = useLogin()
  return user && user.isAdmin
}

export function AdminOnly (props = {}) {
  const isAdmin = useIsAdmin()
  if (!isAdmin) return null
  return props.children
}
