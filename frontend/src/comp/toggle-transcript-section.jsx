import React from 'react'
import { Box, Text, Button, Flex } from '@chakra-ui/react'
import { FaChevronRight, FaChevronDown } from 'react-icons/fa'
import { useTranslation } from 'react-i18next'

import { PostTranscript } from './transcript'

function hasTranscript (post) {
  if (!post.media || !post.media.length) return false
  return post.media.filter(m => m.transcript).length > 0
}

export function ToggleTranscriptSection (props = {}) {
  const { t } = useTranslation()
  const { post } = props
  const [show, setShow] = React.useState(false)

  if (!hasTranscript(post)) return null
  return (
    <Box>
      <Button onClick={() => setShow(show => !show)}>
        <Flex w='140px' direction='row' justify='space-between'>
          {show ? <Text>{t('transcript.hide', 'Hide transcript')}</Text> : <Text>{t('transcript.show', 'Show transcript')}</Text>}
          {show ? <Box ml='4px'><FaChevronDown /></Box> : <Box ml='4px'><FaChevronRight /></Box>}
        </Flex>
      </Button>
      {show && <PostTranscript post={post} />}
    </Box>
  )
}
