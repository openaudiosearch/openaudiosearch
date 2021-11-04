import React from 'react'

import {
  Switch,
  Route
} from 'react-router-dom'

import JobsPage from './pages/jobs'
import SearchPage from './pages/search'
import ImporterPage from './pages/importer'
import LandingPage from './pages/landing-page'
import { PostPage } from './pages/post'
import ImprintPage from './pages/imprint'
import AboutPage from './pages/about-page'

export default function Routes () {
  return (
    <Switch>
      <Route path='/jobs'>
        <JobsPage />
      </Route>
      <Route path='/about'>
        <AboutPage />
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
      <Route path='/imprint'>
        <ImprintPage />
      </Route>
      <Route path='/'>
        <LandingPage />
      </Route>
    </Switch>
  )
}
