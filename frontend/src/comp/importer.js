import React, { useState } from 'react'
import { DragDropContext, Droppable, Draggable } from 'react-beautiful-dnd'

import useSWR from 'swr'
import {
  Flex, Stack, Box, Text, Spacer, Heading, SimpleGrid, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert, Container,
  Table,
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

import fetch from '../lib/fetch'

export default function JobPage (props) {
  const [selectedJobId, setSelectedJobId] = useState(null)
  return (
    <Stack>
      <Heading mb='2'>RSS Importer</Heading>
      <ImportUrl onJobSubmit={setSelectedJobId} />
    </Stack>
  )
}

// TODO: Derive from openapi spec
// const { data, error } = useSWR('openapi.json')
function ImportUrl (props) {
  const { handleSubmit, errors, register } = useForm()
  const { handleSubmit1, errors1, register1 } = useForm()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState(null)
  const [schemaFields, setSchemaFields] = useState(null)
  const [feedFields, setFeedFields] = useState(null)
  const [url, setUrl] = useState(null)
  return (
    <Box p='4' border='1px solid black'>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Heading fontSize='lg'>Import RSS-Feed</Heading>
        <Flex alignContent='end'>
          <FormControl>
            <FormLabel>Media URL</FormLabel>
            <Input name='rss_url' ref={register()} placeholder='https://...' minW='40rem' />
          </FormControl>
          <Flex direction='column' justifyContent='end'>
            <Button type='submit' isLoading={isSubmitting}>Start</Button>
          </Flex>
        </Flex>
      </form>
      <Mapper url={url} schemaFields={schemaFields} feedFields={feedFields} />
    </Box>
  )

  async function onSubmit (values) {
    setIsSubmitting(true)
    try {
      const res = await fetch('/add_new_feed', {
        method: 'POST',
        body: values
      })
      setSchemaFields(res.schema)
      setFeedFields(res.feed_keys)
      setUrl(res.url)
      setIsSubmitting(false)
      console.log('RES', res)
    } catch (err) {
      setIsSubmitting(false)
      setError(err)
      console.log('ERR', err.data)
    }
  }
}

function Mapper (props) {
  const { url, schemaFields, feedFields } = props
  const { handleSubmit, errors, register } = useForm()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState(null)
  return (
    <Box p='4' border='1px solid black'>
      <p>{url}</p>
      <form onSubmit={handleSubmit(onMappingSubmit)}>
        {schemaFields && schemaFields.map((field, i) => (
          <Flex key={i}>
            <Box w='10em' m='0.5em' background='blue' color='white' p='1em'> {field} </Box>
            <Box m='0.5em'>
              <Select name={field} ref={register()} placeholder='Select Feed field'>
                {feedFields && feedFields.map((field, k) => (
                  <option key={k} background='red' color='white' p='1em'> {field} </option>
                ))}
              </Select>
            </Box>
          </Flex>
        ))}
        {schemaFields && <Button type='submit' isLoading={isSubmitting}>save mapping</Button>}
      </form>
    </Box>
  )

  async function onMappingSubmit (values) {
    setIsSubmitting(true)
    try {
      const res = await fetch('/set_mapping', {
        method: 'POST',
        body: { mapping: values, rss_url: url }
      })
      setIsSubmitting(false)
      console.log('RES', res)
    } catch (err) {
      setIsSubmitting(false)
      setError(err)
      console.log('ERR', err.data)
    }
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

function Loading (props) {
  return <Spinner />
}
