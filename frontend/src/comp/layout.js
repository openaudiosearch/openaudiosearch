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
import SearchPage2 from './reactive_search'
import ImporterPage from './importer'
import LandingPage from './landing_page'

export default function Layout (props = {}) {
  const playerHeight = '15rem'
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
      <Link to='/'>
      <Heading p='2' fontSize='xl' mr='4'>Open Audio Search</Heading>
      </Link>
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
        <NavLink to='/importer'>Importer</NavLink>
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
      <Route path='/search/:query'>
        <SearchPage2 />
      </Route>
      <Route exact path='/search'>
        <SearchPage2 />
      </Route>
      <Route path='/importer'>
        <ImporterPage />
      </Route>
      <Route path='/'>
        <LandingPage />
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
