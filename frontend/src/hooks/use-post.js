import useSWR from 'swr'

export function usePost (postId) {
  const { data, error } = useSWR('/post/' + postId)
  return { post: data, error }
}
