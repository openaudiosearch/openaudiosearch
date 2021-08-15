import React from 'react'
import { Flex, Box, Text, Icon, Center, Divider, Link as ChakraLink } from '@chakra-ui/react'
import {
  Link
} from 'react-router-dom'
import { FaGithub } from 'react-icons/fa'

import { useTranslation } from 'react-i18next'
import { Login } from './login'

function FooterLinkText (props = {}) {
  const { children } = props
  return (
    <Text color='gray.500' _hover={{ color: 'gray.600' }} py='3' pl='6' pr='10' fontSize='sm'>
      {children}
    </Text>
  )
}

export function Footer () {
  const { t } = useTranslation()

  return (
    <Flex direction='column'>
      <Divider color='gray.300' />
      <Center>
        <Box w={['90vw', '80vw', '750px', '750px']}>

          <Flex direction={['column', 'column', 'row', 'row']} justify='start'>
            <ChakraLink to='/imprint' as={Link}>
              <FooterLinkText>
                {t('imprint', 'Imprint')}
              </FooterLinkText>
            </ChakraLink>
            <ChakraLink to='/about' as={Link}>
              <FooterLinkText>
                {t('about', 'About us')}
              </FooterLinkText>
            </ChakraLink>
            <ChakraLink href='https://github.com/openaudiosearch/openaudiosearch' isExternal>
              <Flex>
               <FooterLinkText>
                  <Icon as={FaGithub} fontSize='lg' mr='2' />
                  Source code
               </FooterLinkText>
              </Flex>
            </ChakraLink>
            <FooterLinkText>
              <Login>
                <ChakraLink>
                  {t('login', 'Login')}
                </ChakraLink>
              </Login>
             </FooterLinkText>
          </Flex>
        </Box>
      </Center>
    </Flex>
  )
}
