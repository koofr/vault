import { RouterProvider, createBrowserRouter } from 'react-router-dom';

import { mainUnauthenticated } from './mainWebUnauthenticated';
import { NotSupportedPage } from './pages/NotSupportedPage';

export const mainNotSupported = async () => {
  const router = createBrowserRouter([
    {
      path: '*',
      element: <NotSupportedPage />,
    },
  ]);

  await mainUnauthenticated(<RouterProvider router={router} />);
};
