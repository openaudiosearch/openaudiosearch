import React from 'react'

import {
  Switch,
  Route
} from 'react-router-dom'

import JobsPage from './jobs'
import SearchPage from './search'
import ImporterPage from './importer'
import LandingPage from './landing-page'
import { PostPage } from './post'

export function Routes () {
  return (
    <Switch>
      <Route path='/jobs'>
        <JobsPage />
      </Route>
      <Route path='/search/:query'>
        <SearchPage />
      </Route>
      <Route exact path='/search'>
        <SearchPage />
      </Route>
      <Route path='/importer'>
        <ImporterPage />
      </Route>
      <Route path='/post/:postId'>
        <PostPage />
      </Route>
      <Route path='/'>
        <LandingPage />
      </Route>
    </Switch>
  )
}
