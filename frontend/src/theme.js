import { extendTheme } from '@chakra-ui/react'

export default function createTheme (props = {}) {
  // const config = {
  //   initialColorMode: 'light',
  //   useSystemColorMode: false
  // }

  const theme = extendTheme({
    // config,
    fonts: {
      heading: 'Inter',
      body: 'Inter'
    },
    colors: {
      primary: '#260A4A',
      secondary: {
        700: '#4619ae',
        600: '#4619ae',
        500: '#9672EB',
        300: '#CEA8FF',
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
        screen: '#F7FAFC'
      },
      highlightMark: '#edf'
      // brand: {
      //   100: "#f7fafc",
      //   // ...
      //   900: "#1a202c",
      // },
    },
    components: {
      Link: {
        baseStyle: {
          color: 'secondary.600'
        }
      }
    }
  })
  // console.log('theme', theme)
  return theme
}
