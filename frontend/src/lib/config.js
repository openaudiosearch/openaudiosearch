const API_HOST = window.OAS_HOST ||
  window.location.protocol +
  '//' + window.location.hostname +
  (window.location.port ? ':' + window.location.port : '')

export const API_ENDPOINT = window.OAS_URL || API_HOST + '/api/v1'
