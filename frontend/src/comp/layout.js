import React from 'react'
import { Flex, Stack, Box, Heading } from '@chakra-ui/react'
import {
  BrowserRouter as Router,
  Switch,
  Route,
  Link,
  useRouteMatch
} from 'react-router-dom'

import { Player } from './player'
import JobsPage from './jobs'
import SearchPage from './search'
import SearchPage2 from './new_search'

export default function Layout (props = {}) {
  const playerHeight = '6rem'
  return (
    <>
      <Stack minHeight='100vh' bg='bg.screen'  mb={playerHeight}>
        <Header />
        <Main flex='1' />
      </Stack>
      <Footer height={playerHeight} />
    </>
  )
}

function Header () {
  return (
    <Flex mb='4' bg='black' color='white'>
      <Heading p='2' fontSize='xl' mr='4'>Open Audio Search</Heading>
      <Navbar />
    </Flex>
  )
}

function Navbar () {
  return (
    <nav>
      <Flex>
        <NavLink to='/search'>Search</NavLink>
        <NavLink to='/jobs'>Jobs</NavLink>
        <NavLink to='/new_search'>Jobs</NavLink>
      </Flex>
    </nav>
  )
}

function NavLink (props) {
  const { to, children } = props
  const match = useRouteMatch(to)
  const activeProps = {
    bg: 'white',
    color: 'black'
  }
  const styleProps = match ? activeProps : {}
  return (
    <Link to={to} >
      <Box p='4' mr='4' _hover={{ color: 'red' }} {...styleProps}>
        {children}
      </Box>
    </Link>
  )
}

function Main (props) {
  return (
    <Box mx='auto' px='8' {...props}>
      <Routes />
    </Box>
  )
}

function Routes () {
  return (
    <Switch>
      <Route path='/jobs'>
        <JobsPage />
      </Route>
      <Route path='/search'>
        <SearchPage />
      </Route>
      <Route path='/new_search'>
        <SearchPage2 />
      </Route>
    </Switch>
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
