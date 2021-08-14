import React from 'react'
import {
  Alert,
  AlertIcon,
  AlertTitle,
  AlertDescription,
} from '@chakra-ui/react'

export function Error (props) {
  const { error } = props
  if (!error) return null
  const message = String(error)
  return (
    <Alert status='error'>
      <AlertIcon />
      {message}
    </Alert>
  )
}

export function Notice (props) {
  const { message } = props
  if (!message) return null
  return (
    <Alert status='info'>
      <AlertIcon />
      {message}
    </Alert>
  )
}
