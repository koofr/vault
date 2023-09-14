import { RouterProvider, createBrowserRouter } from 'react-router-dom';

import { mainUnauthenticated } from './mainUnauthenticated';
import { NotSupportedPage } from './pages/NotSupportedPage';

export const mainNotSupported = () => {
  const router = createBrowserRouter([
    {
      path: '*',
      element: <NotSupportedPage />,
    },
  ]);

  mainUnauthenticated(<RouterProvider router={router} />);
};
