import React from 'react'
import { FaCog, FaPlay, FaTasks, FaExternalLinkAlt, FaChevronLeft } from 'react-icons/fa'
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
  Text
} from '@chakra-ui/react'
import { useParams, useLocation, useHistory } from 'react-router'

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
      color='white'
      bg='secondary.600'
      shadow='md'
      _hover={{
        bg: 'tertiary.600'
      }}
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
  const history = useHistory()
  const location = useLocation()
  let fromSearch = false
  if (location.state) {
    fromSearch = location.state.fromSearch
  }
  if (!post) return null

  const genres = post.genre.map((genre) =>
    <Tag key={genre} mr='1'>{genre}</Tag>
  )
  const creators = post.creator.map((creator) =>
    <Tag key={creator} mr='1'>{creator}</Tag>
  )
  let contributors = []
  if (post.contributor) {
    contributors = post.contributor.map((contributor) =>
      <Tag key={contributor} mr='1'>{contributor}</Tag>
    )
  }

  return (
    <Flex direction='column' maxWidth='750px'>
      {fromSearch &&
        <Flex direction='row' w='100%' mb='2'>
          <Button onClick={() => history.goBack()} size='sm' variant='ghost'>
            <Box mr='2'><FaChevronLeft /></Box>
            <Box>{t('backtosearch', 'Back to search')}</Box>
          </Button>
        </Flex>}
      <Flex direction={['column', 'column', 'row', 'row']} justify='space-between'>
        <Flex direction='column'>
          <Flex direction={['column', 'column', 'row', 'row']} justify='space-between'>
            <Flex direction={['column', 'column', 'row', 'row']}>
              {post.datePublished &&
                <Text fontSize='sm' mr='2'>
                  {Moment(post.datePublished).format('DD.MM.YYYY')}
                </Text>}
              <Box>
                {genres}
              </Box>
            </Flex>
            {post.url &&
              <Link href={post.url} isExternal>
                <Button size='xs'><Box>{t('sourceurl', 'Source URL')}</Box> <Box ml='10px' mb='3px'><FaExternalLinkAlt /></Box></Button>
              </Link>}
          </Flex>
          <Flex direction={['column', 'column', 'row', 'row']} mt='2'>
            <Heading size='md'>{post.headline}</Heading>
            <Flex align='center' justify='center' ml='2'>
              <PostButtons post={post} />
            </Flex>
          </Flex>
        </Flex>
      </Flex>
      <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' mt='2'>
        {post.publisher && <Text mr='2' fontSize='sm'>{t('by', 'by')} {post.publisher}</Text>}
        {post.creator.length > 0 &&
          <Flex direction='row' mr='2'>
            <Text fontSize='sm' mr='1'>{t('creators', 'Creators')}:</Text>
            {creators}
          </Flex>}
        {post.contributor &&
          <Flex direction='row' mr='2'>
            <Text fontSize='sm' mr='1'>{t('contributors', 'Contributors')}:</Text>
            {contributors}
          </Flex>}
      </Flex>

      {post.description &&
        <Box mt='2'>{post.description}</Box>}

      <Flex direction='row' justify='space-between' mt='4'>
        <ToggleTranscriptSection post={post} />
      </Flex>

    </Flex>
  )
}
