import React from 'react'
import ReactDOM from 'react-dom'

import App from './app'

import './lib/i18n'

const el = document.getElementById('root')
document.body.appendChild(el)

ReactDOM.render(<App />, el)
