import React from 'react'
import { ChakraProvider } from '@chakra-ui/react'
import { SWRConfig } from 'swr'
import {
  HashRouter as Router
} from 'react-router-dom'

import createTheme from './theme'
import Layout from './comp/layout'
import { PlayerProvider } from './comp/player'
import fetch from './lib/fetch'

import 'inter-ui/inter.css'

function App (props) {
  return (
    <Wrapper>
      <Layout />
    </Wrapper>
  )
}

export default App

function Wrapper (props) {
  const { children } = props
  const theme = createTheme()

  return (
    <SWRConfig
      value={{ fetcher: fetch }}
    >
      <ChakraProvider theme={theme}>
        <Router>
          <PlayerProvider>
            {children}
          </PlayerProvider>
        </Router>
      </ChakraProvider>
    </SWRConfig>
  )

  // async function fetcher (url, opts) {
  //   // const endpoint = 'http://localhost:8080/oas/v1'
  //   // url = endpoint + url
  //   const res = await fetch(url, opts)
  //   return res.json()
  // }
}
