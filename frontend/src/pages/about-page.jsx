import React, { useState } from 'react'
import { Heading, Center, Image, Box, Link, Flex, FormControl, FormLabel, FormHelperText, Textarea, Input, Text, Alert, AlertIcon } from '@chakra-ui/react'
import prototypeLogo from '../../assets/PrototypeFund-P-Logo.svg'
import cbaLogo from '../../assets/cba_logo.svg'
import arsoLogo from '../../assets/arso.png'
import netideeLogo from '../../assets/netidee-Projekte-Logo.jpg'

export default function AboutPage () {
  return (
    <Center>
      <Box w={['90vw', '80vw', '750px', '750px']}>
        <Flex ml='6' direction='column'>
          <Heading as='h2' fontSize='xl' my='4'>About</Heading>
          <Text>
            Open Audio Search is an open source project by <Link href="https://arso.xyz" isExternal>arso collective</Link> and <Link href="https://cba.fro.at/" isExternal>cba - community broadcasting archive</Link>.
          </Text>
          <Flex py='8' justify='center'>
            <Link title='arso collective' display='block' mr='8' href='https://arso.xyz/' isExternal>
              <Image src={arsoLogo} w='100px' />
            </Link>
            <Link tile='CBA' display='block' href='https://cba.fro.at/' isExternal>
              <Image src={cbaLogo} w='100px' />
            </Link>
          </Flex>
          <Heading as='h2' fontSize='xl' my='4'>Documentation</Heading>
          <Text>
            Read about how to run your own instance, the technical details, and how to contribute to the project in our <Link fontWeight='bold' href="https://github.com/openaudiosearch/openaudiosearch/blob/main/README.md" isExternal>README</Link>. The README also has links to other resources.
          </Text>
          <Text mt='2'>
            Documentation for the HTTP API is autogenerated and available right from within this instance. You can use this API page to directly make API requests. Please note that after logging in, you can also directly change data through these API requests, so use with care.
          </Text>
          <Text fontWeight='bold' mt='2'>
            <Link href='/swagger-ui' isExternal>Browse HTTP API docs</Link>
          </Text>
          <Flex direction='column' my='4'>
            <Heading as='h3' fontSize='xl' my='4'>About</Heading>
            <Text>
    If you find bugs, want to propose features or share ideas please either open an issue in our public <Link href='https://github.com/openaudiosearch/openaudiosearch/issues/new/choose' isExternal>issue tracker</Link>, say hi on our open <Link href='https://discord.openaudiosearch.org' isExternal>Discord chat server</Link> or send us an <Link href='mailto:info@arso.xyz'>email</Link>.
            </Text>
          </Flex>
          <Heading as='h3' fontSize='xl' my='4'>Funding</Heading>
          <Text>
          The development of Open Audio Search has been supported by
          </Text>
          <Flex direction='row' align='end' my='4' justify='center'>
            <Box mr='10'>
              <Link href='https://prototypefund.de/' isExternal>
                <Image src={prototypeLogo} w='80px' />
              </Link>
            </Box>
            <Box>
              <Link href='https://www.netidee.at/' isExternal>
                <Image src={netideeLogo} w='200px' />
              </Link>
            </Box>
          </Flex>
        </Flex>
      </Box>
    </Center>
  )
}

function FeedbackForm () {
  const [name, setName] = useState()
  const [email, setEmail] = useState()
  const [text, setText] = useState()
  const [success, setSuccess] = useState(false)

  const handleSubmit = (e) => {
    e.preventDefault()
    console.log(name, email, text)
    setSuccess(true)
  }

  return (
    <form onSubmit={handleSubmit}>
      <Flex direction='column' maxWidth='700px'>
        {success &&
          <Alert status='success' mt='2'>
            <AlertIcon />
          Feedback sent! Thank you!
          </Alert>}
        <Text mt='2'>Please help us improve Open Audio Search by providing feedback on your usage, ideas and bugs you may encounter.</Text>
        <FormControl id='email' mt='2'>
          <FormLabel>Name</FormLabel>
          <Input type='text' value={name} onChange={(e) => setName(e.target.value)} />
          <FormHelperText>Not required.</FormHelperText>
        </FormControl>
        <FormControl id='email' mt='2'>
          <FormLabel>Email address</FormLabel>
          <Input type='email' value={email} onChange={(e) => setEmail(e.target.value)} />
          <FormHelperText>Not required. We'll only use your email address to contact you in regards to questions to your feedback.</FormHelperText>
        </FormControl>
        <FormControl id='email' mt='2'>
          <FormLabel>Feedback</FormLabel>
          <Textarea value={text} onChange={(e) => setText(e.target.value)} isRequired />
          <FormHelperText>Please provide some feedback</FormHelperText>
        </FormControl>
        <FormControl mt='2'>
          <Input type='submit' value='Submit' variant='filled' bg='secondary.500' color='white' _hover={{ bg: 'secondary.200' }} />
          <FormHelperText>We will never share your data with third parties.</FormHelperText>
        </FormControl>
      </Flex>
    </form>
  )
}