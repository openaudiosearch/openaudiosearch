import React, { useState } from 'react'
import ReactJson from 'react-json-view'
import Moment from 'moment'
import useSWR from 'swr'
import {
  Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert,
  Table, Thead, Tbody, Tfoot, Tr, Th, Td, TableCaption,
  Drawer, DrawerBody, DrawerFooter, DrawerHeader, DrawerOverlay, DrawerContent, DrawerCloseButton
} from '@chakra-ui/react'
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
      <Flex>
        <Box w={['100%']}>
          <JobList onSelect={setSelectedJobId} selected={selectedJobId} />
        </Box>
      </Flex>
    </Stack>
  )
}

function JobList (props) {
  const { onSelect, selected } = props
  const { data, error, mutate } = useSWR('/jobs')
  if (error) return <Error error={error} />
  if (!data) return <Loading />
  console.log('JOBS', data)
  return (

    <Box
      mt={2}
    >
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
      <Td> {Moment(job.created_at).fromNow()}</Td>
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
              <Button disabled={successDelete} onClick={() => deleteJob(job.id)} colorScheme='red'>Delete Job</Button>
              <Box mr='4' p='2'>
                {error && <Error error={error} />}
                {successDelete &&
                  <Alert status='success'>
                    <AlertIcon />
                    Job with Id: {job.id} was deleted
                  </Alert>}
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
