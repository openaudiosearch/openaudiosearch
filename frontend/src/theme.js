import { extendTheme } from '@chakra-ui/react'

export default function createTheme (props = {}) {
  // const config = {
  //   initialColorMode: 'light',
  //   useSystemColorMode: false
  // }

  const theme = extendTheme({
    // config,
    colors: {
      primary: '#260A4A',
      secondary: {
        // 700: '#9672EB',
        600: '#9672EB',
        500: '#9672EB',
        200: '#CEA8FF',
        100: '#CEA8FF',
        50: '#CEA8FF'
      },
      tertiary: {
        600: '#2DCCC2',
        100: '#C4ECEA'
      },
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
