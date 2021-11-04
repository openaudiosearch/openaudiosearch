import React, { useState } from 'react'
import ReactJson from 'react-json-view'
import useSWR from 'swr'
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert } from '@chakra-ui/react'
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
      <Heading mb='2'>Jobs</Heading>
      <ImportUrl onJobSubmit={setSelectedJobId} />
      <Flex>
        <Box w={['100%', '30%']}>
          <JobList onSelect={setSelectedJobId} selected={selectedJobId} />
        </Box>
        <Box w={['100%', '70%']}>
          {selectedJobId && <Job id={selectedJobId} />}
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
  const { data, error } = useSWR('/jobs', { refreshInterval: 1000 })
  // console.debug('JobList', { data, error })
  if (error) return <Error error={error} />
  if (!data) return <Loading />
  console.log('JOBS', data)
  return (
    <Box>
      {data.map(job => (
        <Box key={job.id} my='2' _hover={{ textDecoration: 'underline' }} bg={selected === job.id ? 'blue.300' : undefined}>
          <Link onClick={e => onSelect(job.id)}>{job.id}</Link>
        </Box>
      ))}
    </Box>
  )
}

function Job (props) {
  const { id } = props
  const { data, error } = useSWR('/job/' + id, { refreshInterval: 1000 })
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
