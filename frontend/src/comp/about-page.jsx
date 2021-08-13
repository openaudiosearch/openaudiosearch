import React from 'react'
import { Heading, Image, Box, Link, Flex, FormControl, FormLabel, FormHelperText, Textarea, Input, Text, Button } from '@chakra-ui/react'
import prototypeLogo from '../../assets/PrototypeFund-P-Logo.svg'
import cbaLogo from '../../assets/cba_logo.svg'
import netideeLogo from '../../assets/netidee-Projekte-Logo.jpg'

export default function AboutPage () {
  return (
    <Box>
      <Heading>About</Heading>
      <Heading as='h3' size='md'>Funding</Heading>
      The development of Open Audio Search has been funded in round 9 of the <Link href='https://prototypefund.de/' isExternal>
        Prototype Fund</Link>.
      <Flex direction='row' align='end' justify='end'>
        <Box>
          <Link href='https://prototypefund.de/' isExternal>
          <Image src={prototypeLogo} w='100px' />
          </Link>
        </Box>
        <Box>
          <Link href='https://www.netidee.at/' isExternal>
          <Image src={netideeLogo} w='200px' />
          </Link>
        </Box>
      </Flex>
      <Heading as='h3' size='md'>Partner</Heading>
      <Link href='https://cba.fro.at/' isExternal>
        <Image src={cbaLogo} w='100px' />
      </Link>

      <FeedbackForm />
    </Box>
  )
}

function FeedbackForm () {
  return (
    <Flex direction='column'>
      <Text>Please help us improve Open Audio Search by providing feedback on your usage, ideas and bugs you may encounter.</Text>
      <FormControl id="email">
        <FormLabel>Name</FormLabel>
        <Input type="text" />
        <FormHelperText>Not required.</FormHelperText>
      </FormControl>
      <FormControl id="email">
        <FormLabel>Email address</FormLabel>
        <Input type="email" />
        <FormHelperText>Not required. We'll only use your email address to contact you in regards to questions to your feedback.</FormHelperText>
      </FormControl>
      <FormControl id="email">
        <FormLabel>Feedback</FormLabel>
        <Textarea isRequired></Textarea>
        <FormHelperText>We'll never share your email.</FormHelperText>
      </FormControl>
      <Button>Submit</Button>
    </Flex>

  )
}
