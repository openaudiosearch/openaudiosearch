import React from 'react'
import ReactJson from 'react-json-view'
import { DataSearch, MultiList, DateRange, ReactiveBase, ReactiveList, SelectedFilters } from '@appbaseio/reactivesearch'
import { Heading, Flex, Box, Spinner, Button, Text } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { useParams, Link } from 'react-router-dom'
import Moment from 'moment'
import { FaFilter } from 'react-icons/fa'
import { PostButtons } from './post'
import { TranscriptSnippet } from './transcript'
import { useIsAdmin } from '../hooks/use-login'
import { CgClose } from 'react-icons/cg'
import { useTranslation } from 'react-i18next'

const { ResultListWrapper } = ReactiveList

export default function SearchPage () {
  const [show, setShow] = React.useState(false)
  const toggleMenu = () => setShow(!show)
  const { query } = useParams()
  const queryStr = query || ''
  const decodedquery = decodeURIComponent(queryStr)
  const url = API_ENDPOINT + '/search'
  const facets = ['searchbox', 'genre', 'datePublished', 'publisher', 'creator']
  const filterButtonOpen =
    <Flex direction='row'>
      <FaFilter />
      <Text ml='10px'>
        Show Filter
      </Text>
    </Flex>
  const filterButtonClose =
    <Flex direction='row'>
      <CgClose />
      <Text ml='10px'>
    Hide Filter
      </Text>
    </Flex>
  const { t } = useTranslation()

  return (
    <Flex color='white'>
      <ReactiveBase
        app='oas'
        url={url}
      >
        <Flex direction='column'>
          <Heading mb='2' ml={[null, null, '350px', '350px']}>{t('search', 'Search')}</Heading>
          <Box
            // w={['90vw', '90vw', '600px', '600px']}
            maxWidth='750px'
            ml={[null, null, '350px', '350px']}
            mt='6'
          >
            <DataSearch
              componentId='searchbox'
              dataField={['headline', 'description', 'transcript']}
              fieldWeights={[5, 1]}
              placeholder={t('searchForm.placeholder', 'Search for radio broadcasts')}
              autosuggest
              highlight
              queryFormat='and'
              fuzziness={0}
              react={{
                and: facets.filter(f => f !== 'searchbox')
              }}
              defaultValue={decodedquery}
            />
            <SelectedFilters showClearAll />
          </Box>

          <Flex direction={['column', 'column', 'row', 'row']} justify-content='flex-start'>
            <Button
              aria-label='FilterMenu'
              display={{ base: 'flex', md: 'none' }}
              onClick={toggleMenu}
              // icon={show ? <CgClose /> : <FiMenu />}
              shadow='md'
              mt='20px'
              mb='10px'
            // w={['50px', '50px', null, null]}
            >
              {show ? filterButtonClose : filterButtonOpen}
            </Button>
            <Box
              display={{ base: show ? 'flex' : 'none', md: 'block' }}
              flexBasis={{ base: '100%', md: 'auto' }}
            >
              <Flex
                direction='column'
                w={['250px', '300px', '300px', '300px']}
                mr={[null, null, '50px', '50px']}
                mb={['30px', '30px', null, null]}
              >
                <Box mb='30px'>
                  <MultiList
                    title={t('publisher', 'Publisher')}
                    componentId='publisher'
                    dataField='publisher.keyword'
                    react={{
                      and: facets.filter(f => f !== 'publisher')
                    }}
                  />
                </Box>
                <Box mb='30px'>
                  <MultiList
                    title={t('creator', 'Creator')}
                    componentId='creator'
                    dataField='creator.keyword'
                    react={{
                      and: facets.filter(f => f !== 'creator')
                    }}
                  />
                </Box>
                <Box mb='30px'>
                  <MultiList
                    title={t('genre', 'Genre')}
                    componentId='genre'
                    dataField='genre.keyword'
                    react={{
                      and: facets.filter(f => f !== 'genre')
                    }}
                  />
                </Box>
                <Box mb='30px'>
                  <DateRange
                    componentId='datePublished'
                    dataField='datePublished'
                    title={t('publishingdate', 'Publishing Date')}
                    queryFormat='basic_date_time_no_millis'
                    react={{
                      and: facets.filter(f => f !== 'datePublished')
                    }}
                  />
                </Box>
              </Flex>
            </Box>
            <Flex direction='column'>
              <Flex maxWidth='750px'>
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
                            <ResultItem item={item} key={i} showSnippets />
                          ))
                        }
                      </ResultListWrapper>
                    )
                  }}
                </ReactiveList>
              </Flex>
            </Flex>
          </Flex>
        </Flex>
      </ReactiveBase>
    </Flex>
  )
}

export function ResultItem (props) {
  const { item, showSnippets } = props
  const isAdmin = useIsAdmin()
  const { t } = useTranslation()

  const snippets = (
    <>
      {Object.entries(item.highlight).map(([fieldname, snippets]) => (
        <SnippetList key={fieldname} fieldname={fieldname} snippets={snippets} post={item} />
      ))}
    </>
  )

  const postId = item.$meta.id

  return (
    <Flex
      direction='column'
      border='2px'
      p='2'
      borderRadius='20px'
      borderColor='gray.200'
      boxShadow='md'
      my='3'
    >
      <Flex
        direction={['column', 'column', 'row', 'row']}
        justify='space-between'
        ml='3'
        mr='3'
      >
        <Flex direction='column' mb='2'>
          <Link to={'post/' + postId}>
            <Heading
              size='md' my={4}
              dangerouslySetInnerHTML={{
                __html: item.headline
              }}
            />
          </Link>
          <div>
            {item.publisher && <div>{t('by', 'by')} {item.publisher}</div>}
            {item.datePublished &&
              <span>
                {t('publishedon', 'published on')}: {Moment(item.datePublished).format('DD.MM.YYYY')}
              </span>}
            <div>{item.description}</div>
          </div>
          {showSnippets && snippets && <div>{snippets}</div>}
          {isAdmin && <ReactJson src={item} collapsed name={false} />}
        </Flex>
        <Flex ml={[null, null, 4, 4]} mt={[4, 4, null, null]} align='center' justify='center'>
          <PostButtons post={item} />
        </Flex>
      </Flex>
    </Flex>
  )
}

function SnippetList (props = {}) {
  const { fieldname, snippets, post } = props
  return (
    <Box p={2}>
      <em>{fieldname}: &nbsp;</em>
      {snippets.map((snippet, i) => (
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
    <Box
      display='inline' css='mark { background: rgba(255,255,0,0.3) }'
      dangerouslySetInnerHTML={{
        __html: props.children
      }}
    />
  )
}
