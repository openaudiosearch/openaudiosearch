import React from 'react'
import { Flex, Box, Text, Button, Center } from '@chakra-ui/react'
import { jsx, css } from '@emotion/core'
import {
  Link,
  useRouteMatch
} from 'react-router-dom'
import { FaBars } from 'react-icons/fa'

import { useTranslation } from 'react-i18next'
import logo from '../../assets/oas_logo-5.svg'

import { Login } from './login'
import { useIsAdmin } from '../hooks/use-login'

const BADGE_STYLE = css`
  position: absolute;
  // background: #ffaacc;
  background: rgba(255,255,255,0.7);
  color: #000;
  font-size: 12px;
  line-height: 12px;
  padding: 5px 3px;
  transform: rotate(-22.5deg);
  border-radius: 10px;
  width: 140px;
  text-align: center;
  left: -36px;
  top: 4px;
  pointer-events: none;
`

function DemoBadge (props) {
  return (
    <Box css={BADGE_STYLE}>alpha demo</Box>
  )
}

export function Header () {
  const [show, setShow] = React.useState(false)
  const toggleMenu = () => setShow(!show)
  const hamburgerButton =
    <Flex>
      <FaBars />
    </Flex>
  return (
    <Flex
      mb='4'
      bg='primary'
      color='white'
      direction='row'
      justify={['space-between', 'start', 'start', 'start']}
    >
      <Flex direction={['column', 'row', 'row', 'row']}>
        <Box w='200px' px='4' py='4' pb={['0', '2', '2', '2']} align='left' position='relative'>
          <DemoBadge />
          <Link to='/'>
            <img src={logo} />
          </Link>
        </Box>
        <Box
          display={[show ? 'flex' : 'none', 'block', 'block', 'block']}
          flexBasis={{ base: '100%', md: 'auto' }}
        >
          <Navbar />
        </Box>
      </Flex>
      <Box align='right'>
        <Button
          aria-label='NavBarMenu'
          display={['flex', 'none', 'none', 'none']}
          onClick={toggleMenu}
          // icon={show ? <CgClose /> : <FiMenu />}
          bg='primary'
          color='white'
          m='6'
        >
          {hamburgerButton}
        </Button>
      </Box>
    </Flex>
  )
}

function Navbar () {
  const { t } = useTranslation()
  const isAdmin = useIsAdmin()

  return (
    <nav>
      <Flex
        py={['0', '2', '2', '2']}
        direction={['column', 'row', 'row', 'row']}
      >
        <NavLink
          exact
          to='/'
        >
          <Text fontSize='lg' fontWeight='bold'>
            {t('discover', 'Discover')}
          </Text>
        </NavLink>
        <NavLink to='/search'>
          <Text fontSize='lg' fontWeight='bold'>
            {t('search', 'Search')}
          </Text>
        </NavLink>
        <NavLink to='/about'>
          <Text fontSize='lg' fontWeight='bold'>
            {t('about', 'About')}
          </Text>
        </NavLink>
        {/* <NavLink to='/jobs'>
          <Text fontSize='xl' fontWeight='bold'>
            {t('jobs', 'Jobs')}
          </Text>
        </NavLink> */}
        {isAdmin &&
          <NavLink to='/importer'>
            <Text fontSize='lg' fontWeight='bold'>
              {t('importer', 'Importer')}
            </Text>
          </NavLink>}
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
    borderBottom: '2px solid',
    borderColor: 'white',
    color: 'white'
  }
  const styleProps = match ? activeProps : { ...activeProps, borderColor: 'primary', color: 'white' }
  const hoverProps = {
    ...activeProps,
    borderColor: 'secondary.300',
    color: 'secondary.300'
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
