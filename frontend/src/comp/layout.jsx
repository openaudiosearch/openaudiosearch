import React from 'react'
import { Stack, Box } from '@chakra-ui/react'

import { Player, usePlayer } from './player'
import { Header } from './header'
import { Footer } from './footer'

export default function Layout (props) {
  const { children } = props
  const { track } = usePlayer()
  const footerHeight = track ? '6rem' : 0
  return (
    <>
      <Stack
        minHeight='100vh'
        bg='bg.screen'
        mb={footerHeight}
      >
        <Header />
        <Main flex='1'>
          {children}
        </Main>
        <Footer />
      </Stack>
      <PlayerFooter height={footerHeight} />
    </>
  )
}

function Main (props) {
  const { children, ...other } = props
  return (
    <Box mx='auto' px='8' {...other}>
      {children}
    </Box>
  )
}

function PlayerFooter (props) {
  const { height = '10rem' } = props
  return (
    <Box
      position='fixed'
      bottom='0'
      left='0'
      right='0'
      height={height}
      bg='primary'
      color='white'
      zIndex='50'
    >
      <Player />
    </Box>
  )
}
