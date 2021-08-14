import React from 'react'
import { Flex, Box, Text, Textarea, Center, Divider, Heading } from '@chakra-ui/react'
import {
  Link
} from 'react-router-dom'

import { useTranslation } from 'react-i18next'
import { Login } from './login'

export default function ImprintPage () {
  const { t } = useTranslation()

  return (
    <Center>
      <Box w={['90vw', '80vw', '750px', '750px']}>
        <Flex direction='column' ml='6'>
          <Heading my='8'>{t('imprint', 'Imprint')}</Heading>
          <Heading size='md' py='6'>{t('contact', 'Contact')}</Heading>
          <Text>Open Audio Search</Text>
          <Text>Moreira Veit Heinzmann Schumann GbR</Text>
          <Text>Sid's Adresse right here 42</Text>
          <Text>791312 Freiburg</Text>
        </Flex>
      </Box>
    </Center>
  )
}
