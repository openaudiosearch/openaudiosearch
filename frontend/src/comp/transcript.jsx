import { Box } from '@chakra-ui/react'
import React from 'react'
import { usePlayer, usePlayerRegionIfPlaying } from './player'

export function TranscriptSnippet (props) {
  const { post, snippet } = props

  const { setTrack, setMark, setPost } = usePlayer()

  const words = React.useMemo(() => parseTranscript(snippet), [snippet])
  const [didPlay, setDidPlay] = React.useState(false)

  const mark = React.useMemo(() => {
    const firstMeta = words.filter(word => word.start !== undefined)[0]
    const { id, start } = firstMeta
    const lastMeta = [...words].reverse().filter(word => word.end !== undefined)[0]
    const { end } = lastMeta
    const sentence = words.map(word => word.word).join(' ')
    return { id, start, end, sentence }
  }, [words])

  const track = post.media[Number(mark.id)]

  const style = {
    display: 'inline-block',
    border: '1px solid #eee',
    padding: '2px 5px',
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
    <Box onClick={onClick} style={style} position='relative'>
      {renderedWords}
      <TranscriptPlayingOverlay track={track} mark={mark} />
    </Box>
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
    backgroundColor: 'rgba(255, 0, 255, 0.15)',
    zIndex: 10,
    width: (percentPlaying * 100) + '%',
    transition: 'width linear .5s'
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
  if (highlightWord) {
    style.background = 'yellow'
  }
  return <span style={style}>{word}&nbsp;</span>
}

function parseTranscript (value) {
  const tokens = value.split(' ')
  return tokens.map((token) => {
    let [word, meta] = token.split('|')
    // skip words that are actually a meta string
    if (word.indexOf(':') !== -1) return null
    let highlightWord = false
    if (word.startsWith('<mark>')) {
      word = word.replace('<mark>', '')
      word = word.replace('</mark>', '')
      highlightWord = true
    }
    const item = { word, highlightWord }
    if (meta) {
      const [start, end, conf, id] = meta.split(':')
      return { ...item, start, end, conf, id }
    } else {
      return item
    }
  }).filter(x => x)
}
