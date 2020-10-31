export const API_V1_PREFIX = '/oas/v1'
export const API_ROOT_PATH = window.OAS_ROOT_PATH || ''
export const API_HOST = process.env.NODE_ENV === 'development'
  ? 'http://localhost:8080'
  : API_ROOT_PATH
export const API_ENDPOINT = API_HOST + API_V1_PREFIX
