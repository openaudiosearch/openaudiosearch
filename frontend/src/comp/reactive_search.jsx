import React from 'react'
import ReactJson from 'react-json-view'
import { DataSearch, ResultList, MultiList, DateRange, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Spacer, Box, Button, Spinner } from '@chakra-ui/react'
import { useParams } from 'react-router-dom'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'
import { TranscriptSnippet } from './transcript'

const { ResultListWrapper } = ReactiveList

export default function SearchPage2 () {
  const { query } = useParams()
  const query_str = query || ""
  const decodedquery = decodeURIComponent(query_str)
  const url = API_ENDPOINT + '/search'
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
                  defaultValue={decodedquery}
                />
              </Box>
              <ReactiveList
                dataField='dateModified'
                componentId='SearchResults'
                pagination
                // defaultQuery={() => ({
                //   highlight: {
                //     type: 'plain'
                //   }
                // })}
                react={{
                  and: facets
                }}
              >
                {({ data, error, loading, ...args }) => {
                  if (loading) return <Spinner size='xl' />
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
  const { track, setTrack, setPost } = usePlayer()

  const snippets = (
    <>
      {Object.entries(item.highlight).map(([fieldname, snippets]) => (
        <SnippetList key={fieldname} fieldname={fieldname} snippets={snippets} post={item} />
      ))}
   </>
  )

  return (
    <ResultList>
      <ResultList.Content>
        {/* <ResultList.Image src={item.image} /> */}
        <Heading size={'lg'} my={4}>
          <HighlightMark>{item.headline}</HighlightMark>
        </Heading>
        <ResultList.Description>
          <div>
            <div>by {item.creator}</div>
            <div>{item.publisher}</div>
          </div>
          <span>
            published on: {item.datePublished}
          </span>

          {snippets}

          <div>
            {item.media && item.media.length && (
              <Button onClick={() => {
                setTrack(item.media[0])
                setPost(item)
              }}>
                Click to play
              </Button>
            )}
          </div>
          <ReactJson src={item} collapsed={true} name={false} />
        </ResultList.Description>
      </ResultList.Content>
    </ResultList>
  )
}

function SnippetList (props = {}) {
  const { fieldname, snippets, post } = props
  return (
    <Box p={2}>
      <em>{fieldname}: &nbsp;</em>
      {snippets.map((snippet , i) => (
        <Snippet key={i} post={post} fieldname={fieldname} snippet={snippet} />
      ))}
    </Box>
  )
}

function Snippet (props = {}) {
  const { fieldname, snippet, post } = props
  if (fieldname === 'transcript') {
    return (
      <TranscriptSnippet post={post} snippet={snippet} />
    )
  } else {
    return (
      <HighlightMark>{snippet}</HighlightMark>
    )
  }
}

function HighlightMark (props = {}) {
  // TODO: Parse mark and do not use dangerouslySetInnerHTML
  return (
      <Box display='inline' css='mark { background: rgba(255,255,0,0.3) }'
        dangerouslySetInnerHTML={{
          __html: props.children
        }}
      />
  )
}
