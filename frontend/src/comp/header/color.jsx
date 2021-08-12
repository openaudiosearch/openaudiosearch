import React from 'react'
import { useColorMode, IconButton } from '@chakra-ui/react'
import { FaMoon, FaSun } from 'react-icons/fa'

export function ColorModeButton () {
  const { colorMode, toggleColorMode } = useColorMode()
  const label = `Change to ${colorMode === 'light' ? 'dark' : 'light'} color mode`
  const icon = colorMode === 'light' ? <FaMoon /> : <FaSun />
  return (
    <IconButton aria-label={label} icon={icon} onClick={toggleColorMode} />
  )
}
