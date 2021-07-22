import React from 'react'
import ReactJson from 'react-json-view'
import { DataSearch, ResultList, ResultCard, MultiList, DateRange, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Spacer, Box, Button, Spinner, Center } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'
import { useHistory } from 'react-router-dom'

const { ResultCardsWrapper } = ReactiveList


export default function LandingPage() {
  const url = API_ENDPOINT + '/search'
  const [value, setValue] = React.useState("")
  const history = useHistory()
  console.log(url)
  return (
    <Flex color='white' align='center'>
      <ReactiveBase
      app='oas'
      url={url}
      >
      <Center>
        <Flex direction='column' align='center'>
          <Box w='800px'>
            <Center>
              <Flex direction='column'>
                <Heading as="h1" size="3xl" mb='7'>Open Audio Search</Heading>
                <Heading as="h2" size="md">The community radio search engine</Heading>
              </Flex>
            </Center>
            <Center>
            <Box w='600px' mt='6'>
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
            <Box>
              <Heading as="h4" size="md" mt={20} mb={5}>Discover</Heading>
            </Box>
            <ReactiveList
                dataField='dateModified'
                componentId='DiscoverItems'
                pagination
                showResultStats={false}
                size={6}
              >
                {({ data, error, loading, ...args }) => {
                  if (loading) return <Spinner size='xl' />
                  // console.log('reactive result', { data, error, loading, args })
                  return (
                    <ResultCardsWrapper>
                      {
                        data.map((item, i) => (
                          <DiscoverItem item={item} key={i} />
                        ))
                      }
                    </ResultCardsWrapper>
                  )}}
              </ReactiveList>
          </Box>
        </Flex>
      </Center>
      </ReactiveBase>
    </Flex>
  )
}

function DiscoverItem (props) {
  const { item } = props
  const { setTrack } = usePlayer()
  return (
    <ResultCard>
        <ResultCard.Title>
          <Heading as="h5" size="sm"
            className="broadcast-title"
            dangerouslySetInnerHTML={{
              __html: item.headline
            }}
          />
        </ResultCard.Title>
        <ResultCard.Description>
          <div>
            <div>by {item.publisher}</div>
          <span>
            published on: {item.datePublished}
          </span>
          </div>
          <div>
            <Button onClick={() => setTrack(item)}>
              Click to play
            </Button>
          </div>
        </ResultCard.Description>
    </ResultCard>
  )
}
