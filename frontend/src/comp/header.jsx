import React from 'react'
import { Flex, Box, Heading } from '@chakra-ui/react'
import {
  Link,
  useRouteMatch
} from 'react-router-dom'

import { Login } from './login'

export function Header () {
  return (
    <Flex mb='4' bg='black' color='white'>
      <Link to='/'>
        <Heading p='4' fontSize='xl' mr='4'>
          Open Audio Search
        </Heading>
      </Link>
      <Navbar />
      <Box flex={1} />
      <Box py={4}>
        <Login />
      </Box>
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
    <Link to={to}>
      <Box
        p='4'
        mr='4'
        _hover={{ color: 'red' }}
        {...styleProps}
      >
        {children}
      </Box>
    </Link>
  )
}
