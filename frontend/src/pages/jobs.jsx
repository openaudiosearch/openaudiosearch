import React, { useState } from 'react'
import ReactJson from 'react-json-view'
import useSWR from 'swr'
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert, 
         Table, Thead, Tbody, Tfoot, Tr, Th, Td, TableCaption,
         Drawer, DrawerBody, DrawerFooter, DrawerHeader, DrawerOverlay, DrawerContent, DrawerCloseButton} from '@chakra-ui/react'
import {
  FaEdit as EditIcon,
  FaCheck as SaveIcon,
  FaSync as RefreshIcon
} from 'react-icons/fa'
import { useForm } from 'react-hook-form'

import fetch from '../lib/fetch'

export default function JobPage (props) {
  const [selectedJobId, setSelectedJobId] = useState(null)
  return (
    <Stack>
      <Heading mb='2'>Jobs</Heading>
      <ImportUrl onJobSubmit={setSelectedJobId} />
      <Flex>
        <Box w={['100%']}>
          <JobList onSelect={setSelectedJobId} selected={selectedJobId} />
        </Box>
      </Flex>
    </Stack>
  )
}

// TODO: Derive from openapi spec
// const { data, error } = useSWR('openapi.json')
function ImportUrl (props) {
  const { handleSubmit, errors, register } = useForm()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState(null)
  const [jobId, setJobId] = useState(null)

  const engines = [{ name: 'Vosk/Kaldi', value: 'vosk' }, { name: 'PyTorch', value: 'pytorch' }]
  return (
    <Box p='4' border='1px solid black'>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Heading fontSize='lg'>Import audio from URL</Heading>
        <Flex alignContent='end'>
          <FormControl mr='2'>
            <FormLabel>Media URL</FormLabel>
            <Input name='media_url' ref={register()} placeholder='https://...' minW='40rem' />
          </FormControl>
          <FormControl mr='2'>
            <FormLabel>Engine</FormLabel>
            <Select name='engine' minW='10rem'>
              {engines.map(engine => <option key={engine.value} value={engine.value}>{engine.name}</option>)}
            </Select>
          </FormControl>
          <Flex direction='column' justifyContent='end'>
            <Button type='submit' isLoading={isSubmitting}>Start</Button>
          </Flex>
          <Box mr='4' p='2'>
            {error && <Error error={error} />}
            {jobId && <Box>Created job: {jobId}</Box>}
          </Box>
        </Flex>
      </form>
    </Box>
  )

  async function onSubmit (values) {
    setIsSubmitting(true)
    try {
      values.engine = 'vosk'
      const res = await fetch('/transcript', {
        method: 'POST',
        body: values
      })
      setJobId(res.id)
      setIsSubmitting(false)
      console.log('RES', res)
    } catch (err) {
      setIsSubmitting(false)
      setError(err)
      console.log('ERR', err.data)
    }
  }
}

function JobList (props) {
  const { onSelect, selected } = props
  const { data, error, mutate } = useSWR('/jobs')
  if (error) return <Error error={error} />
  if (!data) return <Loading />
  console.log('JOBS', data)
  return (
    <Box mt={2}>
      <Flex direction='row-reverse'>
        <Button onClick={() => mutate()} leftIcon={<RefreshIcon />}>Refresh</Button>
      </Flex>
      <Table variant='striped'>
        <Thead>
          <Tr>
            <Th>ID</Th>
            <Th>Status</Th>
            <Th>Created at</Th>
            <Th>Queue</Th>
          </Tr>
        </Thead>
        <Tbody>
          {data.map(job => (
            <Job key={job.id} job={job} />
          ))}
        </Tbody>
      </Table>
    </Box>
  )
}

function Job (props) {
  const { job } = props

  return (
    <Tr>
      <Td>{job.id}</Td>
      <Td>{job.status}</Td>
      <Td>{job.created_at}</Td>
      <Td>{job.queue}</Td>
      <Td><JobDetails job={job} /></Td>
    </Tr>
  )
}

function JobDetails (props) {
  const { job } = props
  const { isOpen, onOpen, onClose } = useDisclosure()
  const btnRef = React.useRef()
  const [error, setError] = useState(null)
  const [successDelete, setSuccessDelete] = useState(false)

  async function deleteJob (id) {
    try {
      const res = await fetch('/job/' + id, {
        method: 'DELETE'
      })
      console.log('RES', res)
      setSuccessDelete(true)
    } catch (err) {
      setError(err)
      console.log('ERR', err.data)
    }
  }

  return (
    <>
      <Button ref={btnRef} onClick={onOpen}>
        Details
      </Button>
      <Drawer
        isOpen={isOpen}
        placement='right'
        onClose={onClose}
        finalFocusRef={btnRef}
      >
        <DrawerOverlay />
        <DrawerContent>
          <DrawerCloseButton />
          <DrawerHeader>Details for Job {job.id}</DrawerHeader>

          <DrawerBody>
            <JobJson id={job.id} />
          </DrawerBody>

          <DrawerFooter>
            <Flex direction='column'>
            <Button onClick={() => deleteJob(job.id)} colorScheme='red'>Delete Job</Button>
              <Box mr='4' p='2'>
              {error && <Error error={error} />}
              {successDelete && <Box>Deleted job: {job.id}</Box>}
              </Box>
            </Flex>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>
    </>
  )
}

function JobJson (props) {
  const { id } = props
  const { data, error } = useSWR('/job/' + id)
  if (error) return <Error error={error} />
  if (!data) return <Loading />
    return <ReactJson src={data} collapsed='3' />
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
