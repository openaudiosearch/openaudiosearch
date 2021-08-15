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

function FooterLink (props) {
  const { link, text, icon, external } = props
  return (
    <ChakraLink href={link} isExternal={external}>
      <Flex>
        <FooterLinkText>
          <Icon as={icon} mr='2' />
          {text}
        </FooterLinkText>
      </Flex>
    </ChakraLink>)
}

export function Footer () {
  const { t } = useTranslation()

  return (
    <Flex direction='column'>
      <Divider color='gray.300' />
      <Center>
        <Box w={['90vw', '80vw', '750px', '750px']}>

          <Flex direction={['column', 'column', 'row', 'row']}>
            <FooterLink
              link='https://github.com/openaudiosearch/openaudiosearch'
              text='Source code'
              icon={FaGithub}
              external
            />
            <FooterLink
              link='https://discord.gg/GjdQjxrPJB'
              text='Join us on discord'
              icon={FaDiscord}
              external
            />
            <FooterLink
              link='/about'
              text={t('about', 'About us')}
              icon={FaRegNewspaper}
            />
            <FooterLink
              link='/imprint'
              text={t('imprint', 'Imprint')}
              icon={FaInfoCircle}
            />
            <FooterLinkText>
              <Login>
                <ChakraLink>
                  <Icon as={FaLock} mr='2' />
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
