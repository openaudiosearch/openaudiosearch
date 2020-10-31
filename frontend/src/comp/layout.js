import React from 'react'
import { Flex, Stack, Box, Heading } from '@chakra-ui/core'

import { Player } from './player'
import Jobs from './jobs'

export default function Layout (props = {}) {
  const playerHeight = '6rem'
  return (
    <>
      <Stack minHeight='100vh' bg='bg.screen'>
        <Header />
        <Main flex='1' mb={playerHeight} />
      </Stack>
      <Footer height={playerHeight} />
    </>
  )
}

function Header () {
  return (
    <>
      <Heading>Open Audio Search</Heading>
    </>
  )
}

function Main (props) {
  return (
    <Box width='720px' mx='auto' p='8' {...props}>
      <Jobs />
    </Box>
  )
}

function Footer (props) {
  const { height = '10rem' } = props
  return (
    <Box
      position='fixed'
      bottom='0'
      left='0'
      right='0'
      height={height}
      bg='black'
      color='white'
    >
      <Player />
    </Box>
  )
}
