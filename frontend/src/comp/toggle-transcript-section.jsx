import React from 'react'
import { Box, Text, Button, Flex } from '@chakra-ui/react'
import { FaChevronRight, FaChevronDown } from 'react-icons/fa'
import { useTranslation } from 'react-i18next'

export function ToggleTranscriptSection(props = {}) {
  const { post } = props
  const [show, setShow] = React.useState(false)
  const toggleTranscript = () => setShow(!show)
  if (!post) return null

  const transcript = post.media.map((media) => {
    if (media.transcript && media.transcript.text)
      return <Text>{media.transcript.text}</Text>
    return null
  })
  const { t } = useTranslation()
  console.log(transcript)

  return (
    <Box>
      {transcript.indexOf(null) < 0 &&
      <Box>
        <Button
          onClick={toggleTranscript}
        >
          <Flex direction="row"> 
            {show ? <FaChevronRight /> : <FaChevronDown />}
            {show ? t('transcript.hide', 'Hide transcript') : t('transcript.show', 'Show transcript')}
          </Flex>
        </Button>
        {show &&
        <Box>
          {transcript}
        </Box>
        }
      </Box>
      }
    </Box>
  )
}
