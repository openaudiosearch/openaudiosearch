import React from 'react'
import { DataSearch, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Box, Spinner, Center, IconButton } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { usePlayer } from './player'
import { Link, useHistory } from 'react-router-dom'
import Moment from 'moment'
import { FaPlay } from 'react-icons/fa'
import { useTranslation } from 'react-i18next'

export default function LandingPage () {
  const url = API_ENDPOINT + '/search'
  const [value, setValue] = React.useState('')
  const history = useHistory()
  const { t } = useTranslation()
  return (
    <Center>
      <Flex color='white' w={['90vw', '90vw', '70vw', '50vw']} align='center' justify='center'>
        <ReactiveBase
          app='oas'
          url={url}
        >
          <Flex direction='column' align='center'>
            <Flex direction='column' align='center'>
              <Flex direction='column' align='left' justify='left'>
                <Heading as='h1' size='2xl' mb='7'>{t('openaudiosearch', 'Open Audio Search')}</Heading>
                <Heading as='h2' size='md'>{t('slogan', 'The community radio search engine')}</Heading>
              </Flex>
              <Center>
                <Box w={['90vw', '80vw', '600px', '600px']} mt='6'>
                  <DataSearch
                    componentId='searchbox'
                    dataField={['headline', 'description', 'transcript']}
                    fieldWeights={[5, 2, 1]}
                    placeholder={t('searchForm.placeholder','Search for radio broadcasts') }
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
                <Heading as='h4' size='md' mt='20' mb='5' ml='5'>{t('discover', 'Discover')}</Heading>
              </Box>
              <ReactiveList
                dataField='datePublished'
                sortBy='desc'
                componentId='DiscoverItems'
                infiniteScroll={false}
                showResultStats={false}
                size={6}
                scrollOnChange={false}
              >
                {({ data, error, loading, ...args }) => {
                  if (loading) return <Spinner size='xl' />
                  return (
                    <Flex direction='column'>
                      {
                        data.map((item, i) => (
                          <DiscoverItem item={item} key={i} />
                        ))
                      }
                    </Flex>
                  )
                }}
              </ReactiveList>
            </Flex>
          </Flex>
        </ReactiveBase>
      </Flex>
    </Center>
  )
}

function DiscoverItem (props) {
  const { item } = props
  const { setTrack, setPost } = usePlayer()
  const postPath = '/post/' + item.$meta.id
  const { t } = useTranslation()
  return (
    <Flex direction='column' border='2px' p='2' borderRadius='20px' borderColor='gray.200' boxShadow='md' my='3'>
      <Flex direction={['column', 'column', 'row', 'row']} justify='space-between' ml='3'>
        <Flex direction='column' mb='2'>
          <Link to={postPath}>
            <Heading
              size='md' my={4}
              dangerouslySetInnerHTML={{
                __html: item.headline
              }}
            />
          </Link>
          <div>
            { item.publisher && <div>{t('by', 'by') } {item.publisher}</div> }
            { item.datePublished &&
              <span>
                {t('publishedon', 'published on')}: {Moment(item.datePublished).format('DD.MM.YYYY')}
              </span>
            }
          </div>
        </Flex>
        <Flex ml={[null, null, 4, 4]} mt={[4, 4, null, null]} align='center' justify='center'>
          <IconButton
            aria-label={t('play', 'Play')}
            color='violet'
            onClick={() => {
              setPost(item)
              setTrack(item.media && item.media[0])
            }}
            icon={<FaPlay />}
            mr='2'
            shadow='md'
          />
        </Flex>
      </Flex>
    </Flex>
  )
}
