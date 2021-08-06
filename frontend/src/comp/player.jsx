import React, { useState, useContext, useMemo, useRef, useCallback, useEffect } from 'react'
import { Box, Flex, IconButton, CircularProgress } from '@chakra-ui/react'
import { WaveSurfer, WaveForm } from 'wavesurfer-react'
import { FaPlay, FaPause, FaUndoAlt, FaRedoAlt } from 'react-icons/fa'

import { API_ENDPOINT } from '../lib/config'

function mediaDataPath (media) {
  if (!media) return null
  const id = media.$meta.id
  return `${API_ENDPOINT}/media/${id}/data`
}

const PlayerContext = React.createContext(null)

export function PlayerProvider (props) {
  const { children } = props

  const [track, setTrack] = useState(null)

  const context = useMemo(() => ({
    track,
    setTrack
  }), [track])

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
  const { track } = usePlayer()
  const rerender = useRerender();
  const [loadingProgress, setLoadingProgress] = useState(0)
  const [ready, setReady] = useState(false)

  const mediaPath = useMemo(() => mediaDataPath(track), [track])

  const wavesurferRef = useRef()
  const handleWSMount = useCallback(
    waveSurfer => {
      wavesurferRef.current = waveSurfer

      wavesurferRef.current.on("ready", () => {
        console.log("WaveSurfer is ready")
        setReady(true)
      })

      wavesurferRef.current.on("region-removed", region => {
        console.log("region-removed --> ", region)
      })

      wavesurferRef.current.on("loading", data => {
        console.log("loading --> ", data)
        setLoadingProgress(data)
      })

      wavesurferRef.current.on("play", () => {
        rerender()
      })

      wavesurferRef.current.on("pause", () => {
        rerender()
      })

      if (window) {
        window.surferidze = wavesurferRef.current
      }
    }, []
  )

  const togglePlay = () => {
    if (!wavesurferRef.current) return
    wavesurferRef.current.playPause()
  }

  const isPlaying = wavesurferRef.current && wavesurferRef.current.isPlaying() 

  const skipBackward = () => {
    if (!wavesurferRef.current) return
    wavesurferRef.current.skipBackward(30)
  }

  const skipForward = () => {
    if (!wavesurferRef.current) return
    wavesurferRef.current.skipForward(30)
  }

  useEffect(() => {
    if (wavesurferRef.current && mediaPath) {
      wavesurferRef.current.load(mediaPath)
    }
  }, [mediaPath, wavesurferRef.current])

  if (!track || !mediaPath) return (
    <Box>Not playing anything</Box>
  )
  console.log('player', { track, ready, loadingProgress })
  let headline = track.headline || track.contentUrl || 'no title'
  // Remove html highlighting tags from title display in player
  headline = headline.replace(/(<([^>]+)>)/gi, "")
  return (
    <Flex direction="column">
      <Box>
        Currently playing: {headline}
      </Box>
      {loadingProgress < 100 && !ready && 
        <Flex align="center" justify="center">
          <CircularProgress value={loadingProgress} color="violet" />
        </Flex>
      }
      <WaveSurfer onMount={handleWSMount}>
        <WaveForm id="waveform" barWidth="3" barHeight="1">
        </WaveForm>
      </WaveSurfer>
      {loadingProgress == 100 && ready &&
        <Flex direction="row">
          <IconButton 
            aria-label="Skip backward"
            color="violet"
            onClick={skipBackward} 
            icon={<FaUndoAlt />}
            mr="2"
          />
          <IconButton 
            aria-label="Play/Pause"
            color="violet"
            onClick={togglePlay} 
            icon={isPlaying ? <FaPause /> : <FaPlay />}
            mr="2"
          />
          <IconButton 
            aria-label="Skip forward"
            color="violet"
            onClick={skipForward} 
            icon={<FaRedoAlt />}
          />
        </Flex>
      }
    </Flex>
  )
}
