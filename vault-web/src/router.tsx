import { createBrowserRouter } from 'react-router-dom';

import { AuthGuardLazy } from './pages/AuthGuardLazy';
import { HomePageLazy } from './pages/HomePageLazy';
import { LandingPageLazy } from './pages/LandingPageLazy';
import { LoginPageLazy } from './pages/LoginPageLazy';
import { NotFoundPageLazy } from './pages/NotFoundPageLazy';
import { NotSupportedPageLazy } from './pages/NotSupportedPageLazy';
import { OAuth2CallbackPageLazy } from './pages/OAuth2CallbackPageLazy';
import { RepoConfigBackupPageLazy } from './pages/RepoConfigBackupPageLazy';
import { RepoCreatePageLazy } from './pages/RepoCreatePageLazy';
import { RepoFilesDetailsPageLazy } from './pages/RepoFilesDetailsPageLazy';
import { RepoFilesPageLazy } from './pages/RepoFilesPageLazy';
import { RepoInfoPageLazy } from './pages/RepoInfoPageLazy';

export const createRouter = () => {
  return createBrowserRouter([
    {
      path: '/',
      element: <AuthGuardLazy />,
      children: [
        {
          index: true,
          element: <HomePageLazy />,
        },
        {
          path: '/repos/create',
          element: <RepoCreatePageLazy />,
        },
        {
          path: '/repos/:repoId',
          element: <RepoFilesPageLazy />,
        },
        {
          path: '/repos/:repoId/details',
          element: <RepoFilesDetailsPageLazy />,
        },
        {
          path: '/repos/:repoId/info',
          element: <RepoInfoPageLazy />,
        },
        {
          path: '/repos/:repoId/configbackup',
          element: <RepoConfigBackupPageLazy />,
        },
      ],
    },
    {
      path: '/oauth2callback',
      element: <OAuth2CallbackPageLazy />,
    },
    {
      path: '/login',
      element: <LoginPageLazy />,
    },
    {
      path: '*',
      element: <NotFoundPageLazy />,
    },
  ]);
};

export const createFallbackRouter = () => {
  return createBrowserRouter([
    {
      path: '/login',
      element: <NotSupportedPageLazy />,
    },
    {
      path: '*',
      element: <LandingPageLazy />,
    },
  ]);
};
