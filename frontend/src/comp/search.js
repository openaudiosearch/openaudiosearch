import React, { useState } from 'react'
import ReactJson from 'react-json-view'
import useSWR from 'swr'
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert } from '@chakra-ui/react'
import {
  FaEdit as EditIcon,
  FaCheck as SaveIcon
} from 'react-icons/fa'

export default function SearchPage (props) {
  const [query, setQuery] = useState(null)
  return (
    <Stack>
      <Heading mb='2'>Search</Heading>
      <Query onSubmit={onSubmit} />
      <ResultsContainer query={query} />
    </Stack>
  )

  function onSubmit (input) {
    console.log('SEARCH', input)
    setQuery(input)
  }
}

function Query (props) {
  const [input, setInput] = useState('')
  const { onSubmit } = props
  return (
    <form onSubmit={onFormSubmit}>
      <Flex>
        <Input
          placeholder='Type here to search'
          onChange={e => setInput(e.target.value)}
        />
        <Button type='submit' disabled={input.length < 3}>Search</Button>
      </Flex>
    </form>
  )

  function onFormSubmit (e) {
    e.preventDefault()
    onSubmit(input)
  }
}


function ResultsContainer (props) {
  const { query } = props
  const queryEnc = encodeURIComponent(query)
  const { data, error } = useSWR(`/search?query=${queryEnc}`, { refreshInterval: 0 })
  return (
    <>
      {data && <Results data={data} />}
    </>
  )
}

function Results (props) {
  const { data } = props
  const total = data.hits.total.value
  const hits = data.hits.hits
  return (
    <Box>
      <strong>{total} results</strong>
      <Box>
        {hits.map((hit, i) => (
          <Hit key={i} hit={hit} />
        ))}
      </Box>
    </Box>
  )
}

function Hit (props) {
  const { hit } = props
  return (
    <Box p='4' borderBottom='1px solid tomato'>
      {hit._source.text}
      <br />
      <em>Score: {hit._score}</em>
      <ReactJson src={hit} collapsed={0} name={false} />
    </Box>
  )
}