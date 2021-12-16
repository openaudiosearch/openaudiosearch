import React, { useState } from 'react'
import { DragDropContext, Droppable, Draggable } from 'react-beautiful-dnd'

import useSWR from 'swr'
import {
  Flex, Stack, Box, Text, Spacer, Heading, SimpleGrid, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert, Container,
  Switch,
  Checkbox,
  Table,
  Tooltip,
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
  TableCaption,
  Tag
} from '@chakra-ui/react'

import {
  FaEdit as EditIcon,
  FaCheck as SaveIcon,
  FaCog as SettingsIcon
} from 'react-icons/fa'
import { useForm } from 'react-hook-form'
import { Notice, Error, LoginRequired } from '../comp/status'

import fetch from '../lib/fetch'
import { useIsAdmin } from '../hooks/use-login'

const DEFAULT_FEED_VALUES = {
  enableAsr: true,
  enableNlp: true
}

export default function FeedPage (props = {}) {
  const isAdmin = useIsAdmin()
  if (!isAdmin) return <LoginRequired />
  return (
    <Stack>
      <Heading>Feeds</Heading>
      <Box maxWidth='60rem' mx='auto' my='2' p='4' bg='white' border='1px' borderColor='gray.200'>
        <CreateFeed />
      </Box>
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
      {rows}
    </>
  )
}

function FeedRow (props = {}) {
  const { feed } = props
  if (!feed) return null
  const settings = toValues(feed)
  return (
    <Flex p={2} m={2} border='1px' borderColor='gray.200' bg='white'>
      <Box py={2} flex={1}>
        URL: <Code>{feed.url}</Code>
        <FeedTag enabled={settings.enableAsr} label='ASR' tooltip='Speech recognition is enabled' />
        <FeedTag enabled={settings.enableNlp} label='NLP' tooltip='Natural language processing is enabled' />
        <Text fontSize='sm'>{feed.$meta.guid}</Text>
      </Box>
      <FeedSettingsModal feed={feed} />
    </Flex>
  )
}

function FeedTag (props = {}) {
  const { enabled = true, label, tooltip, color = 'green' } = props
  if (!enabled) return null
  return (
    <Tooltip label={tooltip}>
      <Tag ml='4' colorScheme={color}>{label}</Tag>
    </Tooltip>
  )
}

function FeedSettingsModal (props) {
  const { feed } = props
  const { isOpen, onOpen, onClose } = useDisclosure()
  return (
    <>
    <Button onClick={onOpen} leftIcon={<SettingsIcon />}>Settings</Button>
      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Feed settings</ModalHeader>
          <ModalCloseButton />
          <ModalBody mb={2}>
            <FeedSettings feed={feed} handleClose={onClose} />
          </ModalBody>
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
        <Switch name='enableAsr' ref={register()} />
        <FormLabel ml='2'>Enable speech recognition</FormLabel>
      </Flex>
      <Flex>
        <Switch name='enableNlp' ref={register()} />
        <FormLabel ml='2'>Enable natural language processing</FormLabel>
      </Flex>
    </FormControl>
  )
}

function FeedSettings (props) {
  const { mutate } = useSWR('/feed')
  const { feed, handleClose } = props
  const defaultValues = React.useMemo(() => toValues(feed), [feed])
  const { handleSubmit, errors, register } = useForm({
    defaultValues
  })
  const formState = useFormState()
  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <Stack>
        <FormState
          mb={4}
          successMessage='Feed settings saved!'
          {...formState}
        />
        <FeedSettingsInner register={register} />
        <Flex>
          <Button type='submit' isLoading={formState.isSubmitting}>
            Save
          </Button>
          <Box flex={1} />
          <Button variant='ghost' onClick={handleClose}>
            Close
          </Button>
        </Flex>
      </Stack>
    </form>
  )

  async function onSubmit (formValues) {
    formState.setIsSubmitting(true)
    try {
      const nextFeed = patchFeed(feed, formValues)
      const res = await fetch('/feed/' + feed.$meta.id, {
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
    enableAsr: feed.mediaJobs?.asr !== undefined,
    enableNlp: feed.postJobs?.nlp !== undefined,
  }
}

function patchFeed (oldFeed, formValues) {
  const feed = { mediaJobs: {}, postJobs: {}, ...oldFeed }
  feed.mediaJobs.asr = formValues.enableAsr ? {} : undefined
  feed.postJobs.nlp = formValues.enableNlp ? {} : undefined
  return feed
}

function FormState (props) {
  let { error, success, setError, setSuccess, successMessage, errorMessage, submitting, setIsSubmitting, ...other } = props
  if (!success && !error) return null
  if (success && !successMessage) successMessage = 'OK!'
  if (error && !errorMessage) errorMessage = 'Error: ' + String(error)
  const status = success ? 'success' : 'error'
  const message = success ? successMessage : errorMessage
  return (
    <Alert status={status} {...other}>
      <AlertIcon />
      {message}
    </Alert>
  )
}

function CreateFeed (props) {
  const { mutate } = useSWR('/feed')
  const { handleSubmit, errors, register } = useForm({
    defaultValues: DEFAULT_FEED_VALUES
  })
  const formState = useFormState()
  const [url, setUrl] = useState(null)
  return (
    <Box>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Heading fontSize='lg' mb='4'>Add new feed</Heading>
        <Stack>
          <FormState successMessage='Feed saved!' {...formState} />
          <FormControl>
            <FormLabel>Feed URL:</FormLabel>
            <Input name='url' ref={register()} placeholder='https://...' minW='40rem' />
          </FormControl>
          <FeedSettingsInner register={register} />
          <Box>
            <Button type='submit' isLoading={formState.isSubmitting}>Save & import</Button>
          </Box>
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

function DeleteFeed (props) {
  const { id } = props
  const { mutate } = useSWR('/feed')
  const { handleSubmit, errors, register } = useForm()
  const formState = useFormState()
  return (
    <Box>
      <form onSubmit={handleSubmit(onSubmit)}>
        <FeedSettingsInner register={register} />
        <Flex direction='column' justifyContent='end'>
          <Button type='submit' isLoading={formState.isSubmitting}>Delete</Button>
        </Flex>
        <FormState successMessage='Feed deleted!' {...formState} />
      </form>
    </Box>
  )

  async function onSubmit (formValues) {
    formState.setIsSubmitting(true)
    try {
      const res = await fetch('/feed/' + id, {
        method: 'DELETE'
      })
      if (res.error) {
        formState.setError(res.error)
      }
      formState.setSuccess(true)
      mutate()
    } catch (err) {
      formState.setError(err)
    }
  }
}
