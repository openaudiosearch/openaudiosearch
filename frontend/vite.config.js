import reactRefresh from '@vitejs/plugin-react-refresh'

const DEFAULT_OAS_URL = 'http://localhost:8080'

export default {
  plugins: [reactRefresh()],
  define: {
    'process.env': {}
  },
  server: {
    proxy: {
      '/api': process.env.OAS_URL || DEFAULT_OAS_URL
    }
  }
}
