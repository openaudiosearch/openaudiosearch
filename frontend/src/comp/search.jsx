import React from 'react'
import ReactJson from 'react-json-view'
import { DataSearch, MultiList, DateRange, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Box, Spinner, Center, IconButton } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'
import { useParams } from 'react-router-dom'
import Moment from 'moment'
import { FaPlay } from 'react-icons/fa'

const { ResultListWrapper } = ReactiveList

export default function SearchPage () {
  const { query } = useParams()
  const queryStr = query || ''
  const decodedquery = decodeURIComponent(queryStr)
  const url = API_ENDPOINT + '/search'
  const facets = ['searchbox', 'genre', 'datePublished', 'publisher', 'creator']
  return (
    <Flex color='white'>
      <ReactiveBase
        app='oas'
        url={url}
      >
        <Flex direction={['column', 'column', 'row', 'row']} justify-content='flex-start'>
          <Flex
            direction='column'
            w={['250px', '300px', '400px', '300px']}
            mr={[null, null, '50px', '50px']}
            mb={['30px', '30px', null, null]}
          >
            <Box mb='30px'>
              <MultiList
                title='Publisher'
                componentId='publisher'
                dataField='publisher.keyword'
                react={{
                  and: facets.filter(f => f !== 'publisher')
                }}
              />
            </Box>
            <Box mb='30px'>
              <MultiList
                title='Creator'
                componentId='creator'
                dataField='creator.keyword'
                react={{
                  and: facets.filter(f => f !== 'creator')
                }}
              />
            </Box>
            <Box mb='30px'>
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
                  )
                }}
              </ReactiveList>
            </Box>
          </Flex>
        </Flex>
      </ReactiveBase>
    </Flex>
  )
}

// function ResultItem (props) {
//   const { item } = props
//   const { track, setTrack } = usePlayer()
//   const highlights = Object.entries(item.highlight).map(([key, value]) => {
//     return (
//       <Box p={2}>
//         <strong>{key}: &nbsp;</strong>
//         {value.map((value, i) => (
//           <span
//             key={i}
//             dangerouslySetInnerHTML={{
//               __html: value
//             }}
//           />
//         ))}
//       </Box>
//     )
//   })

//   return (
//     <ResultList>
//       <ResultList.Content>
//         <Heading
//           size='lg' my={4}
//           dangerouslySetInnerHTML={{
//             __html: item.headline
//           }}
//         />
//         <ResultList.Description>
//           <div>
//             <div>by {item.creator}</div>
//             <div>{item.publisher}</div>
//           </div>
//           <span>
//             published on: {item.datePublished}
//           </span>
//           <>
//             {highlights.map((highlight, i) => (
//               <div key={i}>{highlight}</div>
//             ))}
//           </>
//           <div>
//             {item.media && item.media.length && (
//               <Button onClick={() => setTrack(item.media[0])}>
//                 Click to play
//               </Button>
//             )}
//           </div>
//           <ReactJson src={item} collapsed name={false} />
//         </ResultList.Description>
//       </ResultList.Content>
//     </ResultList>
//   )
// }

function ResultItem (props) {
  const { item } = props
  const { track, setTrack } = usePlayer()
  // const highlights = Object.entries(item.highlight).map(([key, value]) => {
  //   return (
  //     <Box p={2} key={key}>
  //       <strong>{key}: &nbsp;</strong>
  //       {value.map((value, i) => (
  //         <span
  //           key={i}
  //           dangerouslySetInnerHTML={{
  //             __html: value
  //           }}
  //         />
  //       ))}
  //     </Box>
  //   )
  // })
  return (
    <Flex direction='column' border='2px' p='2' borderRadius='20px' borderColor='gray.200' boxShadow='md' my='3'>
      <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' ml='3'>
        <Flex direction='column' mb='2'>
          <Heading
            size='md' my={4}
            dangerouslySetInnerHTML={{
              __html: item.headline
            }}
          />
          <div>
            <div>by {item.creator}</div>
            <div>{item.publisher}</div>
            <span>
            published on: {Moment(item.datePublished).format('DD.MM.YYYY')}
            </span>
            {/* <>
              {highlights.map((highlight, i) => (
                <div key={i}>{highlight}</div>
              ))}
            </> */}
            <div>{item.description}</div>
          </div>
          <ReactJson src={item} collapsed name={false} />
        </Flex>
        <Flex ml={[null, null, 4, 4]} mt={[4, 4, null, null]} align='center' justify='center'>
          <IconButton
            aria-label='Play'
            color='violet'
            onClick={() => setTrack(item)}
            icon={<FaPlay />}
            mr='2'
            shadow='md'
          />
        </Flex>
      </Flex>
    </Flex>
  )
}
