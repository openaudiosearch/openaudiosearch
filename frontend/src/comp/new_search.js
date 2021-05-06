import React from 'react'
import { DataSearch, ResultList, MultiList, CategorySearch, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'

const { ResultListWrapper } = ReactiveList

export default function SearchPage2 () {
  const { track, setTrack } = usePlayer()
  const url = API_ENDPOINT + '/search'
  return (
    <Flex color='white'>
      <ReactiveBase
        app='oas_feed2'
        url={url}
      >


        <Heading mb='2'>Search now</Heading>
        <MultiList
          title='Publisher'
          componentId='publisher'
          dataField='publisher.keyword'
          react={{
            and: ['searchbox', 'genre']
          }}
        />
        <MultiList
          title='Genre'
          componentId='genre'
          dataField='genre.keyword'
          react={{
            and: ['searchbox', 'publisher']
          }}
        />
        <DataSearch
          componentId='searchbox'
          dataField={['headline', 'description']}
          title='Search'
          fieldWeights={[5, 1]}
          placeholder='Search for feeds'
          autosuggest
          highlight
          highlightField='headline'
          queryFormat='and'
          fuzziness={0}
          react={{
            and: ['publisher', 'genre']
          }}
        />

        <ReactiveList
          dataField='dateModified'
          componentId='SearchResults'
          pagination
          react={{
            and: ['publisher', 'searchbox', 'genre']
          }}
        >
          {({ data, error, loading, ...args }) => (
            <ResultListWrapper>
                {
                data.map(item => (
                  <ResultList key={item.identifier}>
                    <ResultList.Content>
                      {/* <ResultList.Image src={item.image} /> */}
                      <ResultList.Title
                        dangerouslySetInnerHTML={{
                          __html: item.headline
                        }}
                      />
                      <ResultList.Description>
                        <div>
                          <div>von {item.creator}</div>
                          <div>{item.publisher}</div>
                        </div>
                        <span>
                            gesendet am: {item.datePublished}
                        </span>
                        <div>
                          <button onClick={() => {
                            setTrack(item)
                          }}
                          >
                        Zum Abspielen Klicken
                          </button>
                        </div>
                      </ResultList.Description>
                    </ResultList.Content>
                  </ResultList>
                ))
              }
              </ResultListWrapper>
          )}
        </ReactiveList>
      </ReactiveBase>
    </Flex>
  )
}
