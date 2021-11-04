import React from 'react'
import { Flex, Box, Text, Textarea, Center, Divider, Heading, Link as ChakraLink } from '@chakra-ui/react'
import {
  Link
} from 'react-router-dom'

import { useTranslation } from 'react-i18next'

export default function ImprintPage () {
  const { t } = useTranslation()

  return (
    <Center>
      <Box w={['90vw', '80vw', '750px', '750px']}>
        <Flex direction='column' ml='6'>
          <Heading size='md' py='8'>{t('contact', 'Contact')}</Heading>
          <Text>See the <ChakraLink as={Link} to='/about'>about page</ChakraLink> for contact information.</Text>
          <Heading size='md' my='8'>{t('imprint', 'Imprint')}</Heading>
          <Imprint />
        </Flex>
      </Box>
    </Center>
  )
}

// TODO: Make this configurable of course.
function Imprint () {
  return (
    <Box>
      Angaben gemäß § 5 TMG
      <br/><br/>
      demo.openaudiosearch.org wird betrieben von
      <br/><br/>
      Cultural Broadcasting Archive<br/>
      Verein zur Förderung digitaler Kommunikation<br/>
      ZVR 1568700466<br/>
      c/o Schanzstraße 18/1502<br/>
      A-1150 Wien
      <br/><br/>
      und
      <br/><br/>
      arso collective<br/>Moreira Veit Heinzmann Schumann GbR <br/> 
      Schauinslandstr. 34<br/> 
      79100 Freiburg <br/> 
      <br/><br/>
      <strong>Haftungsausschluss: </strong><br/><br/><strong>Haftung für Inhalte</strong><br/><br/>
      Die Inhalte unserer Seiten wurden mit größter Sorgfalt erstellt. Für die Richtigkeit, Vollständigkeit und Aktualität der Inhalte können wir jedoch keine Gewähr übernehmen. Als Diensteanbieter sind wir gemäß § 7 Abs.1 TMG für eigene Inhalte auf diesen Seiten nach den allgemeinen Gesetzen verantwortlich. Nach §§ 8 bis 10 TMG sind wir als Diensteanbieter jedoch nicht verpflichtet, übermittelte oder gespeicherte fremde Informationen zu überwachen oder nach Umständen zu forschen, die auf eine rechtswidrige Tätigkeit hinweisen. Verpflichtungen zur Entfernung oder Sperrung der Nutzung von Informationen nach den allgemeinen Gesetzen bleiben hiervon unberührt. Eine diesbezügliche Haftung ist jedoch erst ab dem Zeitpunkt der Kenntnis einer konkreten Rechtsverletzung möglich. Bei Bekanntwerden von entsprechenden Rechtsverletzungen werden wir diese Inhalte umgehend entfernen.
      <br/><br/>
      <strong>Haftung für Links</strong>
      <br/><br/>
      Unser Angebot enthält Links zu externen Webseiten Dritter, auf deren Inhalte wir keinen Einfluss haben. Deshalb können wir für diese fremden Inhalte auch keine Gewähr übernehmen. Für die Inhalte der verlinkten Seiten ist stets der jeweilige Anbieter oder Betreiber der Seiten verantwortlich. Die verlinkten Seiten wurden zum Zeitpunkt der Verlinkung auf mögliche Rechtsverstöße überprüft. Rechtswidrige Inhalte waren zum Zeitpunkt der Verlinkung nicht erkennbar. Eine permanente inhaltliche Kontrolle der verlinkten Seiten ist jedoch ohne konkrete Anhaltspunkte einer Rechtsverletzung nicht zumutbar. Bei Bekanntwerden von Rechtsverletzungen werden wir derartige Links umgehend entfernen.
      <br/><br/>
    </Box>
  )
}
