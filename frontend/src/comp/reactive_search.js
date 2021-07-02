import React from 'react'
import ReactJson from 'react-json-view'
import { DataSearch, ResultList, MultiList, DateRange, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Spacer, Box, Button, Spinner } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'

const { ResultListWrapper } = ReactiveList

export default function SearchPage2 () {
  const { track, setTrack } = usePlayer()
  const url = API_ENDPOINT + '/search'
  console.log(url)
  const facets = ['searchbox', 'genre', 'datePublished', 'publisher', 'creator']
  return (
    <Flex color='white'>
      <ReactiveBase
        app='oas'
        url={url}
      >
        <Flex direction='row' justify-content='flex-start'>
          <Flex direction='column'>
            <Box w='250px' mr='50px' mb='30px'>
              <MultiList
                title='Publisher'
                componentId='publisher'
                dataField='publisher.keyword'
                react={{
                  and: facets.filter(f => f !== 'publisher')
                }}
              />
            </Box>
            <Box w='250px' mr='50px' mb='30px'>
              <MultiList
                title='Creator'
                componentId='creator'
                dataField='creator.keyword'
                react={{
                  and: facets.filter(f => f !== 'creator')
                }}
              />
            </Box>
            <Box w='250px' mr='50px'>
              <MultiList
                title='Genre'
                componentId='genre'
                dataField='genre.keyword'
                react={{
                  and: facets.filter(f => f !== 'genre')
                }}
              />
              <DateRange
                componentId='datePublished'
                dataField='datePublished'
                title='Publishing Date'
                queryFormat='basic_date_time_no_millis'
                react={{
                  and: facets.filter(f => f !== 'datePublished')
                }}
              />
            </Box>
          </Flex>
          <Flex direction='column'>
            <Box w='800px'>
              <Heading mb='2'>Search</Heading>
              <Box w='300px'>
                <DataSearch
                  componentId='searchbox'
                  dataField={['headline', 'description', 'transcript']}
                  title='Search'
                  fieldWeights={[5, 1]}
                  placeholder='Search for feeds'
                  autosuggest
                  highlight
                  queryFormat='and'
                  fuzziness={0}
                  react={{
                    and: facets.filter(f => f !== 'searchbox')
                  }}
                />
              </Box>
              <ReactiveList
                dataField='dateModified'
                componentId='SearchResults'
                pagination
                react={{
                  and: facets
                }}
              >
                {({ data, error, loading, ...args }) => {
                  if (loading) return <Spinner size='xl' />
                  // console.log('reactive result', { data, error, loading, args })
                  return (
                    <ResultListWrapper>
                      {
                        data.map((item, i) => (
                          <ResultItem item={item} key={i} />
                        ))
                      }
                    </ResultListWrapper>
                  )}}
              </ReactiveList>
            </Box>
          </Flex>
        </Flex>
      </ReactiveBase>
    </Flex>
  )
}

function ResultItem (props) {
  const { item } = props
  const highlights = Object.entries(item.highlight).map(([key, value]) => {
    return (
      <Box p={2}>
        <strong>{key}: &nbsp;</strong>
        {value.map((value, i) => (
          <span 
            key={i}
            dangerouslySetInnerHTML={{
              __html: value
            }}
          />
        ))}
      </Box>
    )
  })

  return (
    <ResultList>
      <ResultList.Content>
        {/* <ResultList.Image src={item.image} /> */}
        <Heading size={'lg'} my={4}
            dangerouslySetInnerHTML={{
              __html: item.headline
            }}
        />
        <ResultList.Description>
          <div>
            <div>by {item.creator}</div>
            <div>{item.publisher}</div>
          </div>
          <span>
            published on: {item.datePublished}
          </span>
          <>
            {highlights.map((highlight, i) => (
              <div key={i}>{highlight}</div>
            ))}
          </>
          <div>
            <Button onClick={() => setTrack(item)}>
              Click to play
            </Button>
          </div>
          <ReactJson src={item} collapsed={true} name={false} />
        </ResultList.Description>
      </ResultList.Content>
    </ResultList>
  )
}
