import React from 'react'
import ReactJson from 'react-json-view'
import { DataSearch, ResultList, ResultCard, MultiList, DateRange, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Spacer, Box, Button, Spinner, Center } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'

const { ResultCardsWrapper } = ReactiveList


export default function LandingPage() {
  const url = API_ENDPOINT + '/search'
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
            <Heading as="h3" size="lg" mb='7'>Open Audio Search - <br/>the community radio search engine</Heading>
            </Center>
            <Center>
            <Box w='600px'>
            {/* TODO: Searchbar als Form nachbauen mit handleSubmit mit redirect auf reactive_search Route mit String als default value*/}
              {/* <DataSearch
                componentId='searchbox'
                dataField={['headline', 'description', 'transcript']}
                fieldWeights={[5, 1]}
                placeholder='Search for radio broadcasts'
                autosuggest
                highlight
                queryFormat='and'
                fuzziness={0}
              /> */}
            </Box>
            </Center>
            <Box>
              <Heading as="h4" size="md" mt={20} mb={5}>Discover</Heading>
            </Box>
            <ReactiveList
                dataField='dateModified'
                componentId='DiscoverItems'
                pagination
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
  const { track, setTrack } = usePlayer()
  return (
    <ResultCard>
        <ResultCard.Title>
          <div
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