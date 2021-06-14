import React, { useState } from 'react'
import { DragDropContext, Droppable, Draggable } from 'react-beautiful-dnd'

import useSWR from 'swr'
import {
  Flex, Stack, Box, Text, Spacer, Heading, SimpleGrid, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert, Container,
  Table,
  Accordion,
  AccordionItem,
  AccordionButton,
  AccordionPanel,
  AccordionIcon,
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
  const [mapping, setMapping] = useState({})
  const [url, setUrl] = useState(null)
  return (
    <Box>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Heading fontSize='lg'>Import RSS feed</Heading>
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
      {url && (
        <>
          <Collapsible title='Field mapping'>
            <Mapper url={url} schemaFields={schemaFields} feedFields={feedFields} mapping={mapping} />
          </Collapsible>
          <Button mt={4} isLoading={isSubmitting} onClick={onImportFeed}>Import feed</Button>
        </>
      )}
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
      setMapping(res.mapping)
      setIsSubmitting(false)
      console.log('RES', res)
    } catch (err) {
      setIsSubmitting(false)
      setError(err)
      console.log('ERR', err.data)
    }
  }

  async function onImportFeed () {
    try {
      setIsSubmitting(true)
      const res = await fetch('/feed/import', {
        method: 'POST',
        body: {
          rss_url: url
        }
      })
      console.log('import res', res)
    } catch (err) {
      setError(err)
    } finally {
      setIsSubmitting(false)
    }
  }
}

function Mapper (props) {
  const { url, schemaFields, feedFields, mapping } = props
  const { handleSubmit, errors, register } = useForm()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState(null)
  console.log('Mapper', { url, schemaFields, feedFields, mapping })
  return (
    <Box>
      <p>{url}</p>
      <form onSubmit={handleSubmit(onMappingSubmit)}>
        {schemaFields && schemaFields.map((field, i) => (
          <Flex key={i}>
            <Box w='10em' m='0.5em' background='blue' color='white' p='1em'> {field} </Box>
            <Box m='0.5em'>
              <Select name={field} ref={register()} placeholder='Select Feed field'>
                {feedFields && feedFields.map((feedField, k) => (
                  <option key={k} background='red' color='white' p='1em' selected={mapping[field] === feedField} value={feedField}>{feedField}</option>
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

function Collapsible (props) {
  const { title, children } = props
  return (
    <Accordion allowMultiple>
      <AccordionItem>
        <h2>
          <AccordionButton>
            <Box flex="1" textAlign="left">
              {title}
            </Box>
            <AccordionIcon />
          </AccordionButton>
        </h2>
        <AccordionPanel pb={4}>
          {children}
        </AccordionPanel>
      </AccordionItem>
    </Accordion>
  )
}
