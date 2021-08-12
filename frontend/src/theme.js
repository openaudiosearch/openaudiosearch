import { extendTheme } from '@chakra-ui/react'

export default function createTheme (props = {}) {
  const theme = extendTheme({
    colors: {
      main: '#aa00ea',
      bg: {
        screen: '#fff'
      }
      // brand: {
      //   100: "#f7fafc",
      //   // ...
      //   900: "#1a202c",
      // },
    }
  })
  console.log(theme)
  return theme
}
