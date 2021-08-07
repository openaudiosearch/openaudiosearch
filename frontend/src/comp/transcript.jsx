import { Box } from '@chakra-ui/react'
import React from 'react'
import { usePlayer, usePlayerRegionIfPlaying } from './player'

export function TranscriptHighlight (props) {
  const { source, value } = props
  return (
    <Box>
      <strong>Transcript</strong>:
        {value.map((value, i) => (
          <TranscriptSnippet key={i} source={source} value={value} />
        ))}
    </Box>
  )
}

export function TranscriptSnippet (props) {
  const { source, value } = props

  const { setTrack, setMark, setPost } = usePlayer()

  const words = React.useMemo(() => parseTranscript(value), [value])

  const mark = React.useMemo(() => {
    let firstMeta = words.filter(word => word.start !== undefined)[0]
    let { id, start } = firstMeta
    let lastMeta = [...words].reverse().filter(word => word.end !== undefined)[0]
    let { end } = lastMeta
    let sentence = words.map(word => word.word).join(' ')
    return { id, start, end, sentence }
  }, [words])

  const track = source.media[Number(mark.id)]

  const currentTimeIfPlaying = usePlayerRegionIfPlaying({ track, mark })
  const [didPlay, setDidPlay] = React.useState(false)
  const percentPlaying = React.useMemo(() => {
    if (!currentTimeIfPlaying) return didPlay ? 1 : 0
    if (!didPlay) setDidPlay(true)
    const percent = (currentTimeIfPlaying - mark.start) / (mark.end - mark.start)
    return percent
  }, [mark, currentTimeIfPlaying])

  const style = {
    display: 'inline-block',
    border: '1px solid #eee',
    padding: '5px',
    margin: '5px 5px 0 5px',
    cursor: 'pointer',
    borderRadius: '10px',
    background: '#f0f0f0'
  }

  const overlayStyle = {
    position: 'absolute',
    left: '0px',
    top: '0px',
    bottom: '0px',
    backgroundColor: 'rgba(255, 0, 255, 0.2)',
    zIndex: 10,
    width: (percentPlaying * 100) + '%',
    transition: 'width linear .05s'
  }
  const renderedWords = React.useMemo(() => (
    <>
      {words.map((word, i) => <TranscriptWord key={i} {...word} source={source} />)}
    </>
  ), [words])

  return (
    <Box onClick={onClick} style={style} position='relative'>
      {renderedWords}
      <div style={overlayStyle} />
    </Box>
  )

  function onClick (e) {
    setDidPlay(false)
    setTrack(track)
    setPost(source)
    setMark(mark)
  }
}

export function TranscriptWord (props) {
  const { setTrack, setMark } = usePlayer()
  const { highlightWord, word, start, end, conf, id, source } = props
  const alpha = Number(conf)
  const style = {
    display: 'inline-block',
    cursor: 'pointer',
    color: `rgba(0,0,0,${alpha})`,
  }
  if (highlightWord) {
    style.background = 'yellow'
  }
  return <span style={style}>{word}&nbsp;</span>
}

function parseTranscript (value) {
  let tokens = value.split(' ')
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
    let item = { word, highlightWord }
    if (meta) {
      let [start, end, conf, id] = meta.split(':')
      return { ...item, start, end, conf, id }
    } else {
      return item
    }
  }).filter(x => x)
}
