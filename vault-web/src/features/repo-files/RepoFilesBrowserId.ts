import { createContext, useContext } from 'react';

export const RepoFilesBrowserIdContext = createContext<number>(
  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
  undefined as any,
);

export function useRepoFilesBrowserId() {
  return useContext(RepoFilesBrowserIdContext);
}
