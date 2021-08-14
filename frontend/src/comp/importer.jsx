import React, { useState } from 'react'
import { DragDropContext, Droppable, Draggable } from 'react-beautiful-dnd'

import useSWR from 'swr'
import {
  Flex, Stack, Box, Text, Spacer, Heading, SimpleGrid, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert, Container,
  Switch,
  Checkbox,
  Table,
  AlertTitle,
  AlertDescription,
  Code,
  Accordion,
  AccordionItem,
  AccordionButton,
  AccordionPanel,
  AccordionIcon,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalFooter,
  ModalBody,
  ModalCloseButton,
  Thead,
  Tbody,
  Tfoot,
  Tr,
  Th,
  Td,
  TableCaption
} from '@chakra-ui/react'

import {
  FaEdit as EditIcon,
  FaCheck as SaveIcon
} from 'react-icons/fa'
import { useForm } from 'react-hook-form'
import { Notice, Error } from './status'

import fetch from '../lib/fetch'

export default function FeedPage (props) {
  const [selectedJobId, setSelectedJobId] = useState(null)
  return (
    <Stack>
      <CreateFeed />
      <ListFeeds />
    </Stack>
  )
}

function ListFeeds (props = {}) {
  const { data, error } = useSWR('/feed')
  if (error) return <Error error={error} />
  if (!data || !data.length) return <Notice message='No feeds found' />
  const rows = (
    <>
      {data.map((row, i) => (
        <FeedRow key={i} feed={row} />
      ))}
    </>
  )

  return (
    <>
      <Heading>Feeds</Heading>
      {rows}
    </>
  )
}

function FeedRow (props = {}) {
  const { feed } = props
  if (!feed) return null
  const simpleValues = toValues(feed)
  return (
    <Flex p={2} m={2} border='1px' borderColor='gray.200'>
      <FeedSettingsModal feed={feed} />
      <Box ml='2' p='1'>{feed.url}</Box>
      <Box>Transcribe media? <strong>{simpleValues.transcribe ? 'Yes' : 'No'}</strong></Box>
    </Flex>
  )
}

function FeedSettingsModal (props) {
  const { feed } = props
  const { isOpen, onOpen, onClose } = useDisclosure()
  return (
    <>
      <Button onClick={onOpen}>Settings</Button>
      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Feed settings</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <FeedSettings feed={feed} />
          </ModalBody>
          <ModalFooter>
            <Button variant='ghost' onClick={onClose}>
              Close
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    </>
  )
}

function useFormState (props = {}) {
  const [error, _setError] = useState(null)
  const [submitting, setIsSubmitting] = useState(null)
  const [success, _setSuccess] = useState(null)
  return { error, success, setError, setSuccess, submitting, setIsSubmitting }
  function setError (err) {
    _setError(err)
    _setSuccess(null)
    setIsSubmitting(false)
  }
  function setSuccess (value) {
    _setSuccess(value)
    _setError(null)
    setIsSubmitting(false)
  }
}

function FeedSettingsInner (props) {
  const { register } = props
  return (
    <FormControl as='fieldset'>
      <Flex>
        <Switch name='transcribe' ref={register()} />
        <FormLabel ml='2'>Transcribe items</FormLabel>
      </Flex>
    </FormControl>
  )
}

function FeedSettings (props) {
  const { mutate } = useSWR('/feed')
  const { feed } = props
  const defaultValues = React.useMemo(() => toValues(feed), [feed])
  const { handleSubmit, errors, register } = useForm({
    defaultValues
  })
  const formState = useFormState()
  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <FeedSettingsInner register={register} />
      <Button type='submit' isLoading={formState.isSubmitting}>Save</Button>
      <FormState
        successMessage='Feed settings saved!'
        {...formState}
      />
    </form>
  )

  async function onSubmit (formValues) {
    formState.setIsSubmitting(true)
    try {
      const nextFeed = patchFeed(feed, formValues)
      let res = await fetch('/feed/' + feed.$meta.id, {
        body: nextFeed,
        method: 'PUT'
      })
      formState.setSuccess(true)
      mutate()
    } catch (err) {
      formState.setError(err)
    }
  }
}

function toValues (feed) {
  return {
    url: feed.url,
    transcribe: feed.taskDefaults?.media?.asr?.state === 'wanted'
  }
}

function patchFeed (oldFeed, formValues) {
  const feed = { ...oldFeed }
  if (!feed.taskDefaults) feed.taskDefaults = {}
  if (!feed.taskDefaults.media) feed.taskDefaults.media = {}
  if (!feed.taskDefaults.media.asr) feed.taskDefaults.media.asr = {}
  feed.taskDefaults.media.asr.state = formValues.transcribe ? 'wanted' : null
  return feed
}

function FormState (props) {
  let { success, successMessage, errorMessage, error } = props
  if (!success && !error) return null
  if (success && !successMessage) successMessage = 'OK!'
  if (error && !errorMessage) errorMessage = 'Error: ' + String(error)
  const status = success ? 'success' : 'error'
  const message = success ? successMessage : errorMessage
  return (
    <Alert status={status}>
      <AlertIcon />
      {message}
    </Alert>
  )
}

function CreateFeed (props) {
  const { mutate } = useSWR('/feed')
  const { handleSubmit, errors, register } = useForm()
  const formState = useFormState()
  const [url, setUrl] = useState(null)
  return (
    <Box>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Heading fontSize='lg'>Add new feed</Heading>
        <Stack>
          <FormControl>
            <FormLabel>Media URL</FormLabel>
            <Input name='url' ref={register()} placeholder='https://...' minW='40rem' />
          </FormControl>
          <FeedSettingsInner register={register} />
          <Flex direction='column' justifyContent='end'>
            <Button type='submit' isLoading={formState.isSubmitting}>Save & import</Button>
          </Flex>
          <FormState successMessage='Feed saved!' {...formState} />
        </Stack>
      </form>
    </Box>
  )

  async function onSubmit (formValues) {
    formState.setIsSubmitting(true)
    try {
      const initFeed = { url: formValues.url }
      const feed = patchFeed(initFeed, formValues)
      const res = await fetch('/feed', {
        method: 'POST',
        body: feed
      })
      formState.setSuccess(true)
      mutate(feed)
    } catch (err) {
      formState.setError(err)
    }
  }
}
