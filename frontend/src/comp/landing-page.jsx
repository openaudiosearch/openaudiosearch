import React from 'react'
import { DataSearch, ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch'
import { Heading, Flex, Box, Spinner, Center } from '@chakra-ui/react'
import { API_ENDPOINT } from '../lib/config'
import { Link, useHistory } from 'react-router-dom'
import Moment from 'moment'
import { useTranslation } from 'react-i18next'
import { PostButtons } from './post'
import { ResultItem } from './search'

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
              <Heading as='h1' size='2xl' mb='7' color='secondary.600'>{t('openaudiosearch', 'Open Audio Search')}</Heading>
              <Heading as='h2' size='lg'>{t('slogan', 'The community radio search engine')}</Heading>
              <Center>
                <Box w={['90vw', '80vw', '600px', '600px']} mt='6'>
                  <DataSearch
                    componentId='searchbox'
                    dataField={['headline', 'description', 'transcript']}
                    fieldWeights={[5, 2, 1]}
                    placeholder={t('searchForm.placeholder', 'Search for radio broadcasts')}
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
                          <ResultItem item={item} key={i} showSnippets={false} />
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
