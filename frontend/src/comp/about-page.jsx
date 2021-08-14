import React, { useState } from 'react'
import { Heading, Center, Image, Box, Link, Flex, FormControl, FormLabel, FormHelperText, Textarea, Input, Text, Alert, AlertIcon } from '@chakra-ui/react'
import prototypeLogo from '../../assets/PrototypeFund-P-Logo.svg'
import cbaLogo from '../../assets/cba_logo.svg'
import netideeLogo from '../../assets/netidee-Projekte-Logo.jpg'

export default function AboutPage () {
  return (
    <Center>
      <Box w={['90vw', '80vw', '750px', '750px']}>
        <Flex ml='6' direction='column'>
          <Heading>About</Heading>
          <Heading as='h3' size='md' my='4'>Funding</Heading>
          The development of Open Audio Search has been funded in round 9 of the
          <Link href='https://prototypefund.de/' isExternal>
            Prototype Fund
          </Link>.
          <Flex direction='row' align='end' justify='center'>
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
          <Heading as='h3' size='md' my='4'>Partner</Heading>
          <Link href='https://cba.fro.at/' isExternal>
            <Image src={cbaLogo} w='100px' />
          </Link>

          <Flex direction='column' my='4'>
            <Heading as='h3' size='md'>Feedback</Heading>
            <FeedbackForm />
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
