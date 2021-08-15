import React from 'react'
import { Flex, Box, Text, Icon, Center, Divider, Link as ChakraLink } from '@chakra-ui/react'
import {
  Link
} from 'react-router-dom'
import { FaDiscord, FaGithub, FaInfoCircle, FaLock, FaRegNewspaper } from 'react-icons/fa'

import { useTranslation } from 'react-i18next'
import { Login } from './login'

function FooterLinkText (props = {}) {
  const { children } = props
  return (
    <Text color='gray.500' _hover={{ color: 'gray.600' }} py='3' pl='6' pr='10' fontSize='xs'>
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

          <Flex direction={['column', 'column', 'row', 'row']}>
            <ChakraLink href='https://github.com/openaudiosearch/openaudiosearch' isExternal>
              <Flex>
                <FooterLinkText>
                  <Icon as={FaGithub} mr='2' />
                  Source code
                </FooterLinkText>
              </Flex>
            </ChakraLink>
            <ChakraLink href='https://discord.gg/GjdQjxrPJB' isExternal>
              <Flex>
                <FooterLinkText>
                  <Icon as={FaDiscord} mr='2' />
                  Join us on discord
                </FooterLinkText>
              </Flex>
            </ChakraLink>
            <ChakraLink to='/about' as={Link}>
              <FooterLinkText>
                <Icon as={FaRegNewspaper} mr='2' />
                {t('about', 'About us')}
              </FooterLinkText>
            </ChakraLink>
            <FooterLinkText>
              <Login>
                <ChakraLink>
                  <Icon as={FaLock} mr='2' />
                  {t('login', 'Login')}
                </ChakraLink>
              </Login>
            </FooterLinkText>
            <ChakraLink to='/imprint' as={Link}>
              <FooterLinkText>
                <Icon as={FaInfoCircle} mr='2' />
                {t('imprint', 'Imprint')}
              </FooterLinkText>
            </ChakraLink>
          </Flex>
        </Box>
      </Center>
    </Flex>
  )
}
