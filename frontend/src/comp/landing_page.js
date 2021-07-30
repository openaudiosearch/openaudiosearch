import React from 'react'
import { DataSearch, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Spacer, Box, Button, Spinner, Center, IconButton } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'
import { useHistory } from 'react-router-dom'
import Moment from 'moment'
import { FaPlay } from 'react-icons/fa'


export default function LandingPage() {
  const url = API_ENDPOINT + '/search'
  const [value, setValue] = React.useState("")
  const history = useHistory()
  console.log(url)
  return (
    <Center>
    <Flex color='white' w={['90vw', '90vw', '70vw', '50vw']} align='center' justify='center' >
      <ReactiveBase
      app='oas'
      url={url}
      >
      <Flex direction='column' align='center'>
        <Flex direction='column'align='center'>
          <Flex direction='column' align='left' justify='left'>
            <Heading as="h1" size="2xl" mb='7'>Open Audio Search</Heading>
            <Heading as="h2" size="md">The community radio search engine</Heading>
          </Flex>
          <Center>
            <Box w={['90vw', '80vw', '600px', '600px']} mt='6'>
              <DataSearch
                componentId='searchbox'
                dataField={['headline', 'description', 'transcript']}
                fieldWeights={[5, 2, 1]}
                placeholder='Search for radio broadcasts'
                autosuggest
                queryFormat='and'
                fuzziness={0}
                value={value}
                onChange={(value, triggerQuery, event) => {
                  setValue(value)
                }}
                onValueSelected={(value, cause, source) => {
                    const encoded = encodeURIComponent(value)
                    history.push('/search/' + encoded)
                }}
              />
            </Box>
          </Center>
        </Flex>
        <Flex direction='column' align='left'> 
          <Box>
            <Heading as="h4" size="md" mt='20' mb='5' ml='5'>Discover</Heading>
          </Box>
          <ReactiveList
              dataField='datePublished'
              sortBy='desc'
              componentId='DiscoverItems'
              infiniteScroll={false}
              showResultStats={false}
              size={6}
              scrollOnChange={false}
              // sortOptions={[{label:'checkthisout', dataField:'datePublished', sortBy:'desc' }]}
            >
              {({ data, error, loading, ...args }) => {
                if (loading) return <Spinner size='xl' />
                // console.log('reactive result', { data, error, loading, args })
                return (
                  <Flex direction='column'>
                    {
                      data.map((item, i) => (
                        <DiscoverItem item={item} key={i} />
                      ))
                    }
                  </Flex>
                )}}
          </ReactiveList>
        </Flex>
      </Flex>
      {/* </Center> */}
    </ReactiveBase>
  </Flex>
  </Center>
  )
}

function DiscoverItem (props) {
  const { item } = props
  const { setTrack } = usePlayer()
  return (
    <Flex direction='column' border='2px' p='2' borderRadius='20px' borderColor='gray.200' boxShadow='md' my='3'>
      <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' ml='3'>
        <Flex direction='column' mb='2'>
          <Heading size={'md'} my={4}
              dangerouslySetInnerHTML={{
                __html: item.headline
              }}
          />
          <div>
            <div>by {item.publisher}</div>
          <span>
            published on: {Moment(item.datePublished).format('DD.MM.YYYY')}
          </span>
          </div>
        </Flex>
        <Flex ml={[null, null, 4, 4]} mt = {[4, 4, null, null]} align='center' justify='center'>
            <IconButton 
            aria-label="Play"
            color="violet"
            onClick={() => setTrack(item)}
            icon={ <FaPlay /> }
            mr="2"
            shadow='md'
            />
        </Flex>
      </Flex>
    </Flex>
  )
}
