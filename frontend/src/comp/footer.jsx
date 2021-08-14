import React from 'react'
import { Flex, Box, Text, Icon, Center, Divider, Link as ChakraLink } from '@chakra-ui/react'
import {
  Link
} from 'react-router-dom'
import { FaGithub } from 'react-icons/fa'

import { useTranslation } from 'react-i18next'
import { Login } from './login'

export function Footer () {
  const { t } = useTranslation()

  return (
    <Flex direction='column'>
      <Divider color='gray.300' />
      <Center>
        <Box w={['90vw', '80vw', '750px', '750px']}>

          <Flex direction='row' justify='start'>
            <Link to='/imprint'>
              <Text color='gray.400' py='3' pl='6' pr='10' fontSize='sm'>
                {t('imprint', 'Imprint')}
              </Text>
            </Link>
            <Link to='/about'>
              <Text color='gray.400' py='3' pl='6' pr='10' fontSize='sm'>
                {t('about', 'About us')}
              </Text>
            </Link>
            <ChakraLink href='https://github.com/openaudiosearch/openaudiosearch' isExternal>
              <Icon color='gray.400' as={FaGithub} my='15px' ml='6' mr='10' />
            </ChakraLink>
            <Flex ml='6' mr='5'>
              <Login />
            </Flex>
          </Flex>
        </Box>
      </Center>
    </Flex>
  )
}
