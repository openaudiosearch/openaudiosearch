import React from 'react'
import { Flex, Box, Center, Text } from '@chakra-ui/react'
import {
  Link,
  useRouteMatch
} from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import logo from '../../assets/oas_logo-4.png'

import { Login } from './login'

export function Header () {
  const { t } = useTranslation()

  return (
    <Flex mb='4' bg='primary' color='white'>
      <Box w='200px' p='4'>
        <Link to='/'>
          <img src={logo} />
        </Link>
      </Box>
      <Center>
        <Navbar />
      </Center>
      <Box flex={1} />
      <Center>
        <Box py={4} mr='5'>
          <Login />
        </Box>
      </Center>
    </Flex>
  )
}

function Navbar () {
  const { t } = useTranslation()

  return (
    <nav>
      <Flex>
        <NavLink
          exact
          to='/'
          activeClassName='active-menu-item'
        >
          <Text fontSize='xl' fontWeight='bold'>
            {t('discover', 'Discover')}
          </Text>
        </NavLink>
        <NavLink to='/search'>
          <Text fontSize='xl' fontWeight='bold'>
            {t('search', 'Search')}
          </Text>
        </NavLink>
        <NavLink to='/jobs'>
          <Text fontSize='xl' fontWeight='bold'>
            {t('jobs', 'Jobs')}
          </Text>
        </NavLink>
        <NavLink to='/importer'>
          <Text fontSize='xl' fontWeight='bold'>
            {t('importer', 'Importer')}
          </Text>
        </NavLink>
      </Flex>
    </nav>
  )
}

function NavLink (props) {
  const { to, children, exact } = props
  const match = useRouteMatch({
    path: to,
    exact
  })
  const activeProps = {
    borderBottom: '3px solid',
    borderColor: 'white',
    color: 'white'
  }
  const styleProps = match ? activeProps : {}
  const hoverProps = {
    ...activeProps,
    borderColor: 'secondary.600',
    color: 'secondary.600'
  }
  return (
    <Link to={to}>
      <Box
        p='1'
        m='3'
        mr='6'
        _hover={hoverProps}
        {...styleProps}
      >
        {children}
      </Box>
    </Link>
  )
}
