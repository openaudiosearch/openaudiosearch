export const API_V1_PREFIX = '/api/v1'
export const API_ROOT_PATH = window.OAS_ROOT_PATH || ''
export const API_HOST = getApiHost()
export const API_ENDPOINT = API_HOST + API_V1_PREFIX
console.log({
  API_V1_PREFIX,
  API_ROOT_PATH,
  API_HOST,
  API_ENDPOINT
})

function getApiHost () {
  if (process.env.NODE_ENV === 'development') {
    return 'http://localhost:8080'
  } else {
    return window.location.protocol +
      '//' + window.location.hostname +
      (window.location.port ? ':' + window.location.port : '') +
      API_ROOT_PATH
  }
}
