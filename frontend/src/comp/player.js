import React, { useState, useContext, useMemo } from 'react'
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure } from '@chakra-ui/core'
import {
  FaPlay as PlayIcon,
  FaHeart as HeartIcon,
  FaSkull as HateIcon
} from 'react-icons/fa'

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

export function Player (props = {}) {
  const { track } = usePlayer()
  console.log('player', track)
  return (
    <Box>
      {track && (
        <Box>Currently playing: {track.title}</Box>
      )}
      {!track && (
        <Box>Not playing anything</Box>
      )}
    </Box>
  )
}
