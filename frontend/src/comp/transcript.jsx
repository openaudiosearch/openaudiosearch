import { Box } from '@chakra-ui/react'
import React from 'react'
import { usePlayer } from './player'

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
  const { setTrack, setMark, setPost } = usePlayer()
  const { source, value } = props
  console.log('source', source)
  let words = React.useMemo(() => parseTranscript(value), [value])

  let info = React.useMemo(() => {
    let firstMeta = words.filter(word => word.start !== undefined)[0]
    let { id, start } = firstMeta
    let lastMeta = [...words].reverse().filter(word => word.end !== undefined)[0]
    let { end } = lastMeta
    let sentence = words.map(word => word.word).join(' ')
    return { id, start, end, sentence }
  }, [words])

  let sentence = words

  const style = {
    display: 'inline-block',
    border: '1px solid #eee',
    padding: '5px',
    margin: '5px 5px 0 5px',
    cursor: 'pointer',
    borderRadius: '10px',
    background: '#f0f0f0'
  }

  words = words.map((word, i) => <TranscriptWord key={i} {...word} source={source} />)
  return (
    <Box onClick={onClick} style={style}>
      {words}
    </Box>
  )
  function onClick (e) {
    setTrack(source.media[Number(info.id)])
    setPost(source)
    setMark({
      word: info.sentence,
      start: info.start,
      end: info.end,
      id: info.id
    })
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
