import React from 'react'
import { Flex, Box, Heading } from '@chakra-ui/react'
import {
  Link,
  useRouteMatch
} from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { Login } from './login'

export function Header () {
  const { t } = useTranslation()

  return (
    <Flex mb='4' bg='primary' color='white'>
      <Link to='/'>
        <Heading p='4' fontSize='xl' mr='4'>
          {t('openaudiosearch', 'Open Audio Search')}
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
  const { t } = useTranslation()

  return (
    <nav>
      <Flex>
        <NavLink to='/search'>{t('search', 'Search')}</NavLink>
        <NavLink to='/jobs'>{t('jobs', 'Jobs')}</NavLink>
        <NavLink to='/importer'>{t('importer', 'Importer')}</NavLink>
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
        _hover={{ color: 'secondary.100' }}
        {...styleProps}
      >
        {children}
      </Box>
    </Link>
  )
}
