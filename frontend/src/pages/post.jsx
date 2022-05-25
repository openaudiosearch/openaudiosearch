import React from 'react'
import { FaCog, FaPlay, FaTasks, FaExternalLinkAlt, FaChevronLeft, FaCreativeCommons } from 'react-icons/fa'
import {
  IconButton,
  Menu,
  MenuButton,
  MenuList,
  MenuItem,
  Heading,
  Box,
  Flex,
  Link as ChakraLink,
  Button,
  Tag,
  Text,
  Icon,
  Center
} from '@chakra-ui/react'
import { useParams, useLocation, useHistory } from 'react-router'

import { stripHTML } from '../lib/sanitize'
import { usePlayer } from '../comp/player'
import { useIsAdmin } from '../hooks/use-login'
import { usePost } from '../hooks/use-post'
import fetch from '../lib/fetch'
import { useTranslation } from 'react-i18next'
import Moment from 'moment'
import { Link } from 'react-router-dom'
import { Helmet } from 'react-helmet'

import { PostTranscriptSection } from '../comp/transcript'

export function PostTaskMenuButton (props = {}) {
  const { t } = useTranslation()
  const isAdmin = useIsAdmin()
  if (!isAdmin) return null

  const { post } = props

  let mediaId = null
  let mediaGuid = null
  if (post.media && post.media.length) {
    mediaId = post.media[0].$meta.id
    mediaGuid = post.media[0].$meta.guid
  }

  async function onTranscribeClick (_e) {
    if (!mediaId) return
    const req = {
      typ: 'asr',
      args: { media_id: mediaId },
      subjects: [mediaGuid]
    }
    try {
      const res = await fetch(`/job`, {
        method: 'POST',
        body: req
      })
      console.log('Created job', res)
    } catch (err) {
      console.log('Error', err)
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
    <Flex direction='row' justify='space-between'>
      <Flex mr='3'>
        <PostPlayButton post={post} />
      </Flex>
      <Flex>
        <PostTaskMenuButton post={post} />
      </Flex>
    </Flex>
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
  if (!post) return null
  return (
    <>
      <PostPageHelmet post={post} />
      <PostPageInner {...props} post={post} />
    </>
  )
}

export function PostPageHelmet (props) {
  const { post } = props
  const headline = post.headline || post.$meta.id
  const title = `${headline} – Open Audio Search`
  return (
    <Helmet>
      <title>{title}</title>
    </Helmet>
  )
}

export function PostPageInner (props = {}) {
  const { post } = props
  const { t } = useTranslation()
  const history = useHistory()
  const location = useLocation()

  let fromSearch = false
  if (location.state) {
    fromSearch = location.state.fromSearch
  }

  // Trim items and remove empty and duplicate items from list
  let genres = [... new Set(post.genre.filter(function(gen) {
    return gen.length > 0;
  }).map((genre) => genre.trim()))]
  genres =
    <>
      {genres.map((genre) => (
        <SearchTag label={genre} facet='genre' key={genre}/>
      ))}
    </>

  const creators = 
    <>
      {post.creator.map((creator) => (
        <SearchTag label={creator} facet='creator' key={creator}/>
      ))}
    </>

  let contributors = []
  if (post.contributor) {
    contributors = post.contributor.map((contributor) =>
      <SearchTag label={contributor} facet='contributor' key={contributor}/>
    )
  }

  let duration = null
  if (post.media.length > 0) {
    duration = (post.media[0].duration / 60).toFixed() + ' min'
  }
  const description = React.useMemo(() => stripHTML(post.description), [post.description])

  return (
    <Center>
      <Box w={['90vw', '80vw', '750px', '750px']}>
        <Flex direction='column' maxWidth='750px'>
          {fromSearch && (
            <Flex direction='row' w='100%' mb='2'>
              <Button onClick={() => history.goBack()} size='sm' variant='ghost'>
                <Box mr='2'><FaChevronLeft /></Box>
                <Box>{t('backtosearch', 'Back to search')}</Box>
              </Button>
            </Flex>
          )}
          <Box bg='white' borderRadius='md' boxShadow='sm' borderWidth='1px' borderColor='gray.300' p='4'>
            <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' w='100%'>
              <Flex direction='column' w='100%'>
                <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' w='100%'>
                  <Flex direction={['column', 'column', 'row', 'row']} w='100%'>
                    {post.genre.length > 0 &&
                      <Box>
                        {genres}
                      </Box>}
                  </Flex>
                  {post.url &&
                    <ChakraLink href={post.url} isExternal>
                      <Button size='xs'><Box>{t('sourceurl', 'Source URL')}</Box> <Box ml='10px' mb='3px'><FaExternalLinkAlt /></Box></Button>
                    </ChakraLink>}
                </Flex>
                <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' mt='2' w='100%' my='4'>
                  <Heading size='lg'>{post.headline}</Heading>
                  <PostButtons my='4' post={post} />
                </Flex>
              </Flex>
            </Flex>
            <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' mt='2' w='100%'>
              {post.datePublished && (
                <Box fontSize='sm' mr='2'>
                  {Moment(post.datePublished).format('DD.MM.YYYY')}
                </Box>
              )}
              {post.publisher && 
                <Flex direction='row'>
                  <Text mr='2' fontSize='sm'>{t('by', 'by')}</Text>
                  <SearchTag label={post.publisher} facet='publisher'/>
                </Flex>
              }
              {post.creator.length > 0 &&
                <Flex direction='row' mr='2'>
                  <Text fontSize='sm' mr='1'>{t('creators', 'Creators')}:</Text>
                  {creators}
                </Flex>}
              {post.contributor && contributors.length > 0 &&
                <Flex direction='row' mr='2'>
                  <Text fontSize='sm' mr='1'>{t('contributors', 'Contributors')}:</Text>
                  {contributors}
                </Flex>}

                {duration && (
                  <Box flex='1' fontSize='sm'>{duration}</Box>
                )}
            </Flex>

            <Box mt='4'>{description}</Box>
            <Box my='4'>
              <LicenseInfos my='4' post={post} />
            </Box>
            <PostTranscriptSection post={post}/>
          </Box>
        </Flex>
      </Box>
    </Center>
  )
}

export function LicenseInfos (props) {
  const { post, ...other } = props
  if (post.licence === 'by-nc-sa') {
    return (
      <Flex direction='row' {...other}>
        <Center>
          <Box py='4' pr='4'>
            <Icon as={FaCreativeCommons} />
          </Box>
        </Center>
        <Center>
          <ChakraLink
            href='https://creativecommons.org/licenses/by-nc-sa/2.0/de/'
            isExternal
            p='2'
            color='secondary.600'
          >
            Creative Commons BY-NC-SA 2.0 DE
          </ChakraLink>
        </Center>
      </Flex>
    )
  } else {
    return (
      <Text>
        Licence unknown (see <Link href={post.url} isExternal>source</Link> for details)
      </Text>
    )
  }
}

export function SearchTag (props) {
  const {label, facet} = props
  const encoded = encodeURIComponent(label)
  const url = `/search?${facet}=["${encoded}"]`
  return (
    <Link to={url}>
      <Tag key={label} mr='1' _hover={{ bg: 'gray.200' }}>{label}</Tag>
    </Link>
  )
}
