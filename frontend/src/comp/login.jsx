import React from 'react'
import useSWR, { mutate } from 'swr'
import {
  Flex, Stack, Box, Heading, Input, Button, Spinner,
  useDisclosure,
  Alert,
  AlertIcon,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalFooter,
  ModalBody,
  ModalCloseButton
} from '@chakra-ui/react'

import { useForm } from 'react-hook-form'
import fetch from '../lib/fetch'

export function Login (props = {}) {
  const { children } = props
  const { handleSubmit, register, formState, isSubmitting } = useForm()
  const { data, error } = useSWR('/login')
  // if (error) return <Error error={error} />
  if (!data && !error) return <Spinner />
  if (data && data.ok) return <LoginInfo />
  else return <LoginFormModal>{children}</LoginFormModal>
}

function LoginForm (props = {}) {
  const { onSuccess } = props
  const [error, setError] = React.useState(false)
  const { handleSubmit, register, formState, isSubmitting, reset } = useForm()
  return (
    <Stack>
      {error && (
        <Alert status='error'>
          <AlertIcon />
            Invalid username or password.
        </Alert>
      )}
      <form onSubmit={handleSubmit(onSubmit)}>
        <Input ref={register()} name='username' type='text' placeholder='Username...' />
        <Input mt={2} ref={register()} name='password' type='password' placeholder='Password...' />
        <Button mt={2} type='submit' disabled={isSubmitting}>Login</Button>
      </form>
    </Stack>
  )

  async function onSubmit (data) {
    try {
      const res = await fetch('/login', { method: 'POST', body: data })
      if (res.ok) {
        mutate('/login')
        reset()
        if (onSuccess) onSuccess()
      } else {
        setError(true)
      }
    } catch (err) {
      setError(true)
    }
  }
}

function LoginFormModal (props = {}) {
  const { children } = props
  const { isOpen, onOpen, onClose } = useDisclosure()
  function onClick (e) {
    e.preventDefault()
    onOpen()
  }
  let button
  if (children) {
    button = React.cloneElement(children, { onClick: onOpen })
  } else {
    button = <Button variant='link' onClick={onOpen}>Login</Button>
  }

  return (
    <>
      {button}
      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Login</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <LoginForm onSuccess={onClose} />
          </ModalBody>
        </ModalContent>
      </Modal>
    </>
  )
}

function LoginInfo (props = {}) {
  const { data, error } = useSWR('/login')
  if (error) return <Error error={error} />
  if (!data) return <Spinner />
  if (!data.ok) return <Box>Not logged in</Box>
  return (
    <Box>
      Logged in as {data.user.username}
      <LogoutButton ml={4} />
    </Box>
  )
}

function LogoutButton (props = {}) {
  return <Button variant='link' {...props} onClick={onLogout}>Logout</Button>
  async function onLogout (e) {
    const headers = {}
    // TODO: Maybe we want to "clear" basic auth here.
    // headers['authorization'] = 'Basic MDowCg=='
    await fetch('/logout', { method: 'POST', headers })
    mutate('/login')
  }
}

function Error (props) {
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
