import React, { useState, useContext, useMemo, useRef, useCallback, useEffect } from 'react'
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Icon, useDisclosure } from '@chakra-ui/react'
import {
  FaPlay as PlayIcon,
  FaHeart as HeartIcon,
  FaSkull as HateIcon
} from 'react-icons/fa'
import { WaveSurfer, WaveForm } from 'wavesurfer-react'
import { FaPlay, FaPause } from 'react-icons/fa'

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

  const wavesurferRef = useRef()
  const handleWSMount = useCallback(
    waveSurfer => {
      wavesurferRef.current = waveSurfer

      wavesurferRef.current.on("ready", () => {
        console.log("WaveSurfer is ready")
      })

      wavesurferRef.current.on("region-removed", region => {
        console.log("region-removed --> ", region)
      })

      wavesurferRef.current.on("loading", data => {
        console.log("loading --> ", data)
      })

      wavesurferRef.current.on("play", data => {
        rerender()
      })

      wavesurferRef.current.on("pause", data => {
        rerender()
      })

      if (window) {
        window.surferidze = wavesurferRef.current
      }
    }, []
  )

  const togglePlay = useCallback(() => {
    if (!wavesurferRef.current) return
    wavesurferRef.current.playPause()
  }, [])

  const isPlaying = wavesurferRef.current && wavesurferRef.current.isPlaying() 

  useEffect(() => {
    if (wavesurferRef.current && track.contentUrl) {
      wavesurferRef.current.load(track.contentUrl)
      console.log('WaveSurfer loading file')
    }
  }, [track, wavesurferRef.current])

  if (!track || !track.contentUrl) return (
    <Box>Not playing anything</Box>
  )
  console.log('player', track)
  let headline = track.headline || track.contentUrl || 'no title'
  // Remove html highlighting tags from title display in player
  headline = headline.replace(/(<([^>]+)>)/gi, "")
  return (
    <Box>
        <Box>
          Currently playing: {headline}
          <WaveSurfer onMount={handleWSMount}>
            <WaveForm id="waveform">
            </WaveForm>
          </WaveSurfer>
          <IconButton 
            aria-label="Play/Pause"
            color="violet"
            onClick={togglePlay} 
            icon={isPlaying ? <FaPause /> : <FaPlay />}
          />
        </Box>
    </Box>
  )
}
