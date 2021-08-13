import React from 'react'
import { FaCog, FaPlay, FaTasks, FaExternalLinkAlt } from 'react-icons/fa'
import {
  IconButton,
  Menu,
  MenuButton,
  MenuList,
  MenuItem,
  Heading,
  Box,
  Flex,
  Link,
  Button,
  Tag,
  Text,
} from '@chakra-ui/react'
import { useParams } from 'react-router'

import { usePlayer } from './player'
import { useIsAdmin } from '../hooks/use-login'
import { usePost } from '../hooks/use-post'
import fetch from '../lib/fetch'
import { useTranslation } from 'react-i18next'
import Moment from 'moment'

import { ToggleTranscriptSection } from './toggle-transcript-section'

export function PostTaskMenuButton (props = {}) {
  const { t } = useTranslation()
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
        aria-label={t('options', 'Options')}
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
  const { t } = useTranslation()

  if (!post.media || !post.media.length) return null
  return (
    <IconButton
      onClick={onClick}
      aria-label={t('playthispost', 'Play this post')}
      icon={<FaPlay />}
      isRound
      color='secondary.500'
      shadow='md'
      {...rest}
    />
  )

  function onClick (_e) {
    setTrack(post.media[0])
    setPost(post)
  }
}

export function PostPage (props = {}) {
  const { postId } = useParams()
  const { post } = usePost(postId)
  const { t } = useTranslation()
  if (!post) return null
  
  const genres = post.genre.map((genre) => 
    <Tag key={genre}>{genre}</Tag>  
  )
  const creators = post.creator.map((creator) => 
    <Tag key={creator}>{creator}</Tag>  
  )

  return (
    <Flex direction="column" maxWidth='750px'>
      <Flex direction={['column', 'column', 'row', 'row']} justify='space-between'>
        <Flex direction="column">
          <Flex direction="row">
            { post.datePublished &&
            <span>
              {Moment(post.datePublished).format('DD.MM.YYYY')}
            </span>
            }
            <Box ml='2'>
              {genres}
            </Box>
          </Flex>
          <Heading size='md'>{post.headline}</Heading>
        </Flex>
        <Flex align='center' justify='center'>
          <PostButtons post={post} />
        </Flex>
      </Flex>
      <Box>
        {creators}
        {post.url &&
        <Link href={post.url} isExternal>
          <Button>{t('sourceurl', 'Source URL')} <FaExternalLinkAlt mx="2px" /></Button>
        </Link>
        }
      </Box>

      {post.description &&
        <Box>{post.description}</Box>
      }

      <Box mt='2'>
        <ToggleTranscriptSection post={post} />
      </Box>

    </Flex>
  )
}
