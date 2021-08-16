import { chakra, Box, Icon, Tooltip, Divider } from '@chakra-ui/react'
import React from 'react'
import { usePlayer, usePlayerRegionIfPlaying, formatDuration } from './player'
import { FaVolumeUp } from 'react-icons/fa'

export function TranscriptSnippet (props) {
  const { post, snippet } = props

  const { setTrack, setMark, setPost } = usePlayer()

  const [didPlay, setDidPlay] = React.useState(false)

  const mark = React.useMemo(() => {
    return parseTranscriptSentence(snippet)
  }, [snippet])

  const { id, start, end, sentence, words } = mark

  const track = post.media[Number(mark.id)]

  const style = {
    display: 'inline-block',
    border: '1px solid #eee',
    padding: '2px 10px',
    margin: '5px 5px 0 5px',
    cursor: 'pointer',
    borderRadius: '10px'
  }

  const renderedWords = React.useMemo(() => (
    <>
      {words.map((word, i) => <TranscriptWord key={i} {...word} post={post} />)}
    </>
  ), [words])

  return (
    <Tooltip label='Click to play'>
      <Box onClick={onClick} style={style} _hover={{ bg: 'gray.50'}} position='relative'>
        {renderedWords}
        <Icon as={FaVolumeUp} ml='2' mr='2' color='gray.400' />
        <chakra.span fontSize='sm' color='gray.600'>{formatDuration(mark.start)}</chakra.span>
        <TranscriptPlayingOverlay track={track} mark={mark} />
      </Box>
    </Tooltip>
  )

  function onClick (e) {
    setDidPlay(false)
    setTrack(track)
    setPost(post)
    setMark(mark)
  }
}

function TranscriptPlayingOverlay (props) {
  const { track, mark } = props
  const currentTimeIfPlaying = usePlayerRegionIfPlaying({ track, mark })
  const [hasPlayed, setHasPlayed] = React.useState(false)
  const percentPlaying = React.useMemo(() => {
    if (!hasPlayed && currentTimeIfPlaying) setHasPlayed(true)
    if (!currentTimeIfPlaying) return hasPlayed ? 1 : 0
    const percent = (currentTimeIfPlaying - mark.start) / (mark.end - mark.start)
    return percent
  }, [mark, currentTimeIfPlaying])

  const overlayStyle = {
    position: 'absolute',
    left: '0px',
    top: '0px',
    bottom: '0px',
    backgroundColor: 'rgba(0, 0, 0, 0.05)',
    zIndex: 10,
    width: (percentPlaying * 100) + '%'
    // transition: 'width linear .5s'
  }
  return (
    <div style={overlayStyle} />
  )
}

export function TranscriptWord (props) {
  const { setTrack, setMark } = usePlayer()
  const { highlightWord, word, start, end, conf, id } = props
  const alpha = Number(conf)
  const style = {
    display: 'inline-block',
    cursor: 'pointer',
    color: `rgba(0,0,0,${alpha})`
  }
  let bg = 'transparent'
  if (highlightWord) bg = 'highlightMark'
  return <Box as='span' bg={bg} style={style}>{word}&nbsp;</Box>
}

export function parseTranscriptSentence (value) {
  const words = parseTranscriptWords(value)
  const firstMeta = words.filter(word => word.start !== undefined && word.end !== undefined)[0]
  const { id, start } = firstMeta
  const lastMeta = [...words].reverse().filter(word => word.end !== undefined)[0]
  const { end } = lastMeta
  const sentence = words.map(word => word.word).join(' ')
  return { id, start, end, sentence, words }
}

export function parseTranscriptWords (value) {
  const tokens = value.split(' ')
  return tokens.map((token) => {
    let highlightWord = false
    if (token.startsWith('<mark>')) {
      highlightWord = true
    }
    token = token.replace('<mark>', '')
    token = token.replace('</mark>', '')
    let [word, meta] = token.split('|')
    // skip words that are actually a meta string
    if (word.indexOf(':') !== -1) return null
    const item = { word, highlightWord }
    if (meta) {
      const [start, end, conf, id] = meta.split(':')
      return { ...item, start, end, conf, id }
    } else {
      return item
    }
  }).filter(x => x)
}
