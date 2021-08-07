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
  Stack,
} from '@chakra-ui/react'
import { FaPlay, FaPause, FaUndoAlt, FaRedoAlt } from 'react-icons/fa'

import { API_ENDPOINT } from '../lib/config'

function mediaDataPath (media) {
  if (!media) return null
  return media.contentUrl
  // const id = media.$meta.id
  // return `${API_ENDPOINT}/media/${id}/data`
}

function trackHeadline ({ track, post }) {
  if (!post || !track) return null
  let headline = post.headline || track.contentUrl || post.id || null
  // Remove html highlighting tags from title display in player
  headline = headline.replace(/(<([^>]+)>)/gi, "")
  return headline
}

const PlayerContext = React.createContext(null)

export function PlayerProvider (props) {
  const { children } = props

  const [track, setTrack] = useState(null)
  const [mark, setMark] = useState(null)
  const [post, setPost] = useState(null)

  const context = useMemo(() => ({
    track,
    setTrack,
    mark,
    setMark,
    post,
    setPost
  }), [track, mark])

  return (
    <PlayerContext.Provider value={context}>
      {children}
    </PlayerContext.Provider>
  )
}

export function usePlayer () {
  const context = useContext(PlayerContext)
  return context
}

function useRerender() {
  const [rerender, setRerender] = useState(0)
  return function () {
    setRerender((counter) => counter + 1)
  }
}

export function Player (props = {}) {
  const ref = React.useRef()
  const { track, mark, post } = usePlayer()
  const mediaPath = useMemo(() => mediaDataPath(track), [track])
  const { start = 0, end = 0, word = ''} = mark || {}
  const [pos, setPos] = useState(null)
  const [playstate, setPlaystate] = useState({})

  const headline = trackHeadline({ track, post })
  const audio = ref.current

  React.useEffect(() => {
    if (!audio) return
    let pos = 0
    if (mark && mark.start) pos = mark.start
    setPos(pos)
    audio.play()
  }, [ref.current, track, mark])

  React.useEffect(() => {
    if (!audio) return
    audio.currentTime = pos
  }, [pos])

  React.useEffect(() => {
    if (!audio) return
    function updateState (e) {
      const state = {
        playing: !audio.paused,
        canplay: audio.readyState > 2,
        currentTime: Math.floor(audio.currentTime),
        duration: Math.floor(audio.duration) || 0
      }
      setPlaystate(state)
    }
    function throttledTimeUpdate (e) {
      if (Math.floor(audio.currentTime) === playstate.currentTime) return
      updateState(e)
    }
    audio.addEventListener('pause', updateState)
    audio.addEventListener('play', updateState)
    audio.addEventListener('timeupdate', throttledTimeUpdate)
    audio.addEventListener('canplay', updateState)
    audio.addEventListener('durationchange', updateState)
    return () => {
      if (!audio) return
      audio.removeEventListener('pause', updateState)
      audio.removeEventListener('play', updateState)
      audio.removeEventListener('timeupdate', throttledTimeUpdate)
      audio.removeEventListener('canplay', updateState)
      audio.removeEventListener('durationchange', updateState)
    }
  }, [audio])

  function togglePlay (e) {
    if (!audio) return
    if (playstate.playing) audio.pause()
    else audio.play()
  }

  let posPercent = 0
  if (playstate.currentTime) {
    posPercent = playstate.currentTime / playstate.duration
  }

  function setPosPercent (percent) {
    if (!playstate.duration) return
    const nextTime = playstate.duration * percent
    setPos(nextTime)
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
          label="Play/Pause"
          onClick={togglePlay}
          icon={playstate.playing ? <FaPause /> : <FaPlay />}
          disabled={!playstate.canplay}
        />
        <Box p={2}>
          {formatDuration(playstate.currentTime)}
        </Box>
        <Box p={2} flex={1}>
          <Timeslider pos={posPercent} onChange={setPosPercent} />
        </Box>
        <Box p={2}>
          {formatDuration(playstate.duration)}
        </Box>
        <audio style={{ display: 'none' }} ref={ref} src={mediaPath} controls></audio>
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
      aria-label="slider-ex-1"
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
  let s = secs - h * 3600 - m * 60
  if (h) return `${pad(h)}:${pad(m)}:${pad(s)}`
  return `${pad(m)}:${pad(s)}`
}

function pad (num) {
  if (String(num).length === 1) return '0' + num
  else return '' + num
}
