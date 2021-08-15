import React, { useState, useContext, useMemo, useRef, useCallback, useEffect, forwardRef } from 'react'
import {
  Box,
  Flex,
  IconButton,
  Slider,
  SliderTrack,
  SliderFilledTrack,
  SliderThumb,
  Stack,
  Tooltip,
  useSliderContext,
  chakra,
  Icon
} from '@chakra-ui/react'
import { cx } from '@chakra-ui/utils'
import { FaPlay, FaPause } from 'react-icons/fa'
import { RiArrowUpSFill } from 'react-icons/ri'
import { useTranslation } from 'react-i18next'
import { MdGraphicEq } from 'react-icons/md'

import { API_ENDPOINT } from '../lib/config'

import { parseTranscript } from './transcript'

// Get the audio content URL for a media
function mediaContentURL (media) {
  if (!media) return null
  return media.contentUrl
  // const id = media.$meta.id
  // return `${API_ENDPOINT}/media/${id}/data`
}

function trackHeadline ({ track, post }) {
  if (!post || !track) return null
  let headline = post.headline || track.contentUrl || post.id || null
  // Remove html highlighting tags from title display in player
  headline = headline.replace(/(<([^>]+)>)/gi, '')
  return headline
}

/**
 * The player context holds setters and values for the
 * currently playing media track, post, and mark (region).
 */
export const PlayerContext = React.createContext(null)

/**
 * The playstate context holds the audio element and play state (time, play/pause, ...)
 */
const PlaystateContext = React.createContext(null)

/**
 * The player provider provides the player and playstate contexts and renders an (invisible) audio element.
 * It also implements basic logic: Set audio element src on track change, change position on mark change, etc.
 */
export function PlayerProvider (props) {
  const { children } = props

  const [track, setTrack] = useState(null)
  const [mark, setMark] = useState(null)
  const [post, setPost] = useState(null)

  const [lastMark, setLastMark] = useState(null)

  const src = mediaContentURL(track)
  const { audio, element, ...state } = useAudioElement({ src })

  // Jump player position whenever a mark is being set or the track is changed.
  React.useEffect(() => {
    if (!audio || !track) return
    let pos = 0
    if (mark && mark !== lastMark) {
      pos = mark.start
      setLastMark(mark)
    }
    audio.currentTime = pos
    audio.play()
  }, [audio, mark, track])

  const playerContext = useMemo(() => ({
    track,
    setTrack,
    mark,
    setMark,
    post,
    setPost
  }), [track, mark, post])

  const playstateContext = useMemo(() => ({
    ...state,
    audio
  }), [audio, state.currentTime, state.duration, state.playing, state.canplay])

  return (
    <PlayerContext.Provider value={playerContext}>
      <PlaystateContext.Provider value={playstateContext}>
        {element}
        {children}
      </PlaystateContext.Provider>
    </PlayerContext.Provider>
  )
}

/**
 * Use the player context.
 * @return {object} { mark, setMark, track, setTrack, post, setPost }
 */
export function usePlayer () {
  const context = useContext(PlayerContext)
  return context
}

/**
 * Use the playstate context.
 * @return {object} { audio: HTMLMediaElement, currentTime, duration, canplay, playing }
 */
export function usePlaystate () {
  const context = useContext(PlaystateContext)
  return context
}

export function usePlayerRegionIfPlaying ({ track, mark }) {
  const player = usePlayer()
  const { currentTime, audio } = usePlaystate()
  function isActive () {
    return (
      (player.track === track) &&
      (currentTime > mark.start && currentTime < mark.end)
    )
  }
  if (isActive()) return currentTime
  return 0
}

function useRerender () {
  const [rerender, setRerender] = useState(0)
  return function () {
    setRerender((counter) => {
      return counter + 1
    })
  }
}

function useAudioElement (props = {}) {
  const { src } = props
  const ref = React.useRef()
  const rerender = useRerender()

  const element = React.useMemo(() => (
    <audio style={{ display: 'none' }} ref={ref} />
  ), [])

  const audio = ref.current

  React.useEffect(() => {
    if (!audio) return
    audio.src = src
  }, [audio, src])

  React.useEffect(() => {
    if (!audio) return

    // In paused state render whenever the handler is triggered.
    // In playing state update every 500ms
    let interval = null
    function updateState (e) {
      if (audio.paused) {
        if (interval) {
          clearInterval(interval)
          interval = null
        }
        rerender()
      } else if (!interval) {
        interval = setInterval(rerender, 500)
        rerender()
      }
    }

    audio.addEventListener('pause', updateState)
    audio.addEventListener('play', updateState)
    audio.addEventListener('timeupdate', updateState)
    audio.addEventListener('canplay', updateState)
    audio.addEventListener('durationchange', updateState)

    return () => {
      if (interval) clearInterval(interval)
      if (!audio) return
      audio.removeEventListener('pause', updateState)
      audio.removeEventListener('play', updateState)
      audio.removeEventListener('timeupdate', updateState)
      audio.removeEventListener('canplay', updateState)
      audio.removeEventListener('durationchange', updateState)
    }
  }, [audio])

  let state
  if (!audio) {
    state = {
      playing: false,
      canplay: false,
      currentTime: 0,
      duration: 0
    }
  } else {
    state = {
      playing: !audio.paused,
      canplay: audio.readyState > 2,
      currentTime: audio.currentTime || 0,
      duration: audio.duration || 0
    }
  }

  return {
    element,
    audio,
    ...state
  }
}

export function Player (props = {}) {
  const { track, mark, post } = usePlayer()
  const state = usePlaystate()
  const [draggingPos, setDraggingPos] = React.useState(null)
  const { audio } = state

  const { start = 0, end = 0, word = '' } = mark || {}
  const headline = trackHeadline({ track, post })

  const { t } = useTranslation()

  function togglePlay (e) {
    if (!audio) return
    if (state.playing) audio.pause()
    else audio.play()
  }

  let posPercent = 0
  if (state.currentTime) {
    posPercent = state.currentTime / state.duration
  }

  function setPosPercent (percent) {
    if (!state.duration) return
    const nextTime = state.duration * percent
    audio.currentTime = nextTime
  }

  let snippets = []
  if (post && post.highlight && post.highlight.transcript) {
    snippets = post.highlight.transcript.map((snippet) => parseTranscript(snippet))
  }

  const displayTime = draggingPos ? state.duration * draggingPos : state.currentTime

  return (
    <Stack p={2} bg='primary' color='white' zIndex='100'>
      <Flex direction='column'>
        <Box px='3'>
          <strong>{headline || ''}</strong>
        </Box>
        <Flex dir='row'>
          <PlayerButton
            label={state.playing ? t('pause', 'Pause') : t('play', 'Play')}
            onClick={togglePlay}
            icon={<Box pl='1px'>{state.playing ? <FaPause /> : <FaPlay />}</Box>}
            disabled={!state.canplay}
          />
          <Box p={2} w={100}>
            {formatDuration(displayTime)}
          </Box>
          <Box p={2} flex={1}>
            <Timeslider
              duration={state.duration}
              pos={posPercent}
              onChange={setPosPercent}
              snippets={snippets}
              onDraggingChange={setDraggingPos}
            />
          </Box>
          <Box p={2}>
            {formatDuration(state.duration)}
          </Box>
        </Flex>
      </Flex>
    </Stack>
  )
}

function PlayerButton (props = {}) {
  const { label, ...other } = props
  return (
    <IconButton
      aria-label={label}
      color='secondary.500'
      bg='primary'
      isRound
      _hover={{ bg: 'tertiary.100' }}
      // variant='ghost'
      mr={2}
      {...other}
    />
  )
}

function Timeslider (props = {}) {
  const { pos, duration, onChange, onDraggingChange, snippets } = props
  const [dragging, setDragging] = useState(false)
  const [draggingValue, setDraggingValue] = useState(null)

  let value
  if (dragging && draggingValue) value = draggingValue
  else value = pos * 100

  const tooltipStyle = {
    px: "8px",
    py: "2px",
    bg: "var(--tooltip-bg)",
    color: 'white',
    fontSize: 'lg',
    borderRadius: "sm",
    fontWeight: "medium",
    fontSize: "sm",
    boxShadow: "md",
    maxW: "320px",
    zIndex: "1000",
  }

  const displayTime = formatDuration(duration * value / 100)
  return (
    <Slider
      aria-label='slider-ex-1'
      focusThumbOnChange={false}
      value={value}
      onChangeStart={onChangeStart}
      onChangeEnd={onChangeEnd}
      onChange={onSliderChange}
      position='relative'
    >
      <SliderTrack bg='white'>
        <SliderFilledTrack bg='secondary.500' />
      </SliderTrack>
      <SliderThumb boxSize={6}>
        <Box position='relative'>
          <Box color='secondary.500' as={MdGraphicEq} />
          {dragging && (
            <Box {...tooltipStyle} position='absolute' top='-2rem' left='-1rem'>
              {displayTime}
            </Box>
          )}
        </Box>
      </SliderThumb>
      <SliderSnippets snippets={snippets} />
    </Slider>
  )

  function onChangeStart (value) {
    setDragging(true)
  }

  function onChangeEnd (value) {
    if (!dragging) return
    setDragging(false)
    setPlayerPos(value)
    if (onDraggingChange) onDraggingChange(null)
  }

  function setPlayerPos (value) {
    value = (value || 0) / 100
    if (value !== pos) onChange(value)
  }

  function onSliderChange (value) {
    if (dragging) {
      setDraggingValue(value)
      if (onDraggingChange) onDraggingChange(value / 100)
    } else {
      setPlayerPos(value)
    }
  }
}

function formatDuration (secs) {
  if (!secs) secs = 0
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs - h * 3600) / 60)
  const s = Math.floor(secs - h * 3600 - m * 60)
  if (h) return `${pad(h)}:${pad(m)}:${pad(s)}`
  return `${pad(m)}:${pad(s)}`
}

function pad (num) {
  if (String(num).length === 1) return '0' + num
  else return '' + num
}

function SliderSnippets (props = {}) {
  const { snippets } = props
  const state = usePlaystate()
  const { setMark } = usePlayer()
  if (!state.duration) return null

  function onMarkClick (mark) {
    setMark(mark)
  }

  function findStart (parts) {
    for (const snippet of parts) {
      if (snippet.start !== undefined) return Number(snippet.start)
    }
    return 0
  }

  function snippetPosition (parts) {
    return (findStart(parts) / state.duration) * 100
  }

  function snippetLabel (parts) {
    return parts.map((snippet) => snippet.word || '').join(' ')
  }

  return (
    <>
      {snippets.map((snippet, index) => (
        <SliderMark
          display={['none', 'block', 'block', 'block']}
          key={index}
          value={snippetPosition(snippet)}
          onClick={() => onMarkClick(snippet[0])}
          mt='-5px'
        >
          <Tooltip label={snippetLabel(snippet)} placement='top' zIndex='10000'>
            <Box><Icon as={RiArrowUpSFill} color='secondary.600' w='8' h='10' /></Box>
          </Tooltip>
        </SliderMark>
      ))}
    </>
  )
}

export const SliderMark = forwardRef((props, ref) => {
  const { getMarkerProps } = useSliderContext()
  const markProps = getMarkerProps(props, ref)
  markProps.style.pointerEvents = 'all'
  return (
    <chakra.div
      {...markProps}
      className={cx('chakra-slider__marker', props.className)}
    />
  )
})
