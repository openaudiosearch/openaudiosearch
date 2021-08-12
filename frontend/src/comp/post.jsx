import React from 'react'
import { FaCog, FaPlay, FaTasks } from 'react-icons/fa'
import {
  IconButton,
  Menu,
  MenuButton,
  MenuList,
  MenuItem
} from '@chakra-ui/react'
import { useParams } from 'react-router'

import { usePlayer } from './player'
import { useIsAdmin } from '../hooks/use-login'
import { usePost } from '../hooks/use-post'
import fetch from '../lib/fetch'

export function PostTaskMenuButton (props = {}) {
  const isAdmin = useIsAdmin()
  if (!isAdmin) return null

  const { post } = props

  let mediaId = null
  if (post.media && post.media.length) {
    mediaId = post.media[0].$meta.id
  }

  async function onTranscribeClick (_e) {
    if (!mediaId) return
    try {
      const jobId = await fetch(`/task/transcribe-media/${mediaId}`, {
        method: 'POST'
      })
      console.log('Created job: ' + jobId)
    } catch (err) {
    }
  }

  return (
    <Menu>
      <MenuButton
        as={IconButton}
        aria-label='Options'
        icon={<FaCog />}
        variant='outline'
      />
      <MenuList>
        {mediaId && (
          <MenuItem icon={<FaTasks />} onClick={onTranscribeClick}>
            Transcribe medias
          </MenuItem>
        )}
      </MenuList>
    </Menu>
  )
}

export function PostButtons (props = {}) {
  const { post } = props
  if (!post) return null
  return (
    <>
      <PostPlayButton post={post} />
      <PostTaskMenuButton post={post} />
    </>
  )
}

export function PostPlayButton (props = {}) {
  const { post, ...rest } = props
  const { setTrack, setPost } = usePlayer()

  if (!post.media || !post.media.length) return null
  return (
    <IconButton
      onClick={onClick}
      aria-label='Play this post'
      icon={<FaPlay />}
      rounded
      color='violet'
      {...rest}
    />
  )

  function onClick (_e) {
    setTrack(post.media[0])
    setPost(post)
  }
}

export function PostPage (props = {}) {
  const params = useParams()
  console.log('ÜPARAMS', params)
  const { postId } = useParams()
  const post = usePost(postId)
  return (
    <pre>
      {JSON.stringify(post, 0, 2)}
    </pre>
  )
}
