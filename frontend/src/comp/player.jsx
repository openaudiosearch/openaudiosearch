import React, { useState, useContext, useMemo, useRef, useCallback, useEffect } from 'react'
import {
  Box,
  Flex,
  IconButton,
  CircularProgress,
  Slider,
  SliderTrack,
  SliderFilledTrack,
  SliderThumb,
  Stack
} from '@chakra-ui/react'
import { FaPlay, FaPause, FaUndoAlt, FaRedoAlt } from 'react-icons/fa'

import { API_ENDPOINT } from '../lib/config'

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
  const { audio } = state

  const { start = 0, end = 0, word = '' } = mark || {}
  const headline = trackHeadline({ track, post })

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

  return (
    <Stack p={2} bg='black' color='white'>
      <Box px='3'>
        <strong>{headline || ''}</strong>
        &nbsp;
        {word}
      </Box>
      <Flex dir='row'>
        <PlayerButton
          label={state.playing ? 'Pause' : 'Play'}
          onClick={togglePlay}
          icon={<Box pl='1px'>{state.playing ? <FaPause /> : <FaPlay />}</Box>}
          disabled={!state.canplay}
        />
        <Box p={2}>
          {formatDuration(state.currentTime)}
        </Box>
        <Box p={2} flex={1}>
          <Timeslider pos={posPercent} onChange={setPosPercent} />
        </Box>
        <Box p={2}>
          {formatDuration(state.duration)}
        </Box>
      </Flex>
    </Stack>
  )
}

function PlayerButton (props = {}) {
  const { label, ...other } = props
  return (
    <IconButton
      aria-label={label}
      colorScheme='pink'
      isRound
      variant='ghost'
      mr={2}
      {...other}
    />
  )
}

function Timeslider (props = {}) {
  const { pos, onChange } = props
  const [dragging, setDragging] = useState(false)
  const [draggingValue, setDraggingValue] = useState(null)

  let value
  if (dragging && draggingValue) value = draggingValue
  else value = pos * 100
  return (
    <Slider
      aria-label='slider-ex-1'
      focusThumbOnChange={false}
      value={value}
      onChangeStart={onChangeStart}
      onChangeEnd={onChangeEnd}
      onChange={onSliderChange}
      colorScheme='pink'
    >
      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <SliderThumb />
    </Slider>
  )

  function onChangeStart (value) {
    setDragging(true)
  }

  function onChangeEnd (value) {
    setDragging(false)
    setPlayerPos(value)
  }

  function setPlayerPos (value) {
    value = (value || 0) / 100
    onChange(value)
  }

  function onSliderChange (value) {
    if (dragging) {
      setDraggingValue(value)
    } else {
      setPlayerPos(value)
    }
  }
}

function formatDuration (secs) {
  if (!secs) secs = 0
  let h = Math.floor(secs / 3600)
  let m = Math.floor((secs - h * 3600) / 60)
  let s = Math.floor(secs - h * 3600 - m * 60)
  if (h) return `${pad(h)}:${pad(m)}:${pad(s)}`
  return `${pad(m)}:${pad(s)}`
}

function pad (num) {
  if (String(num).length === 1) return '0' + num
  else return '' + num
}
