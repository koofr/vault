import { createContext, useContext } from 'react';

export const RepoFilesBrowserIdContext = createContext<number>(
  undefined as any
);

export function useRepoFilesBrowserId() {
  return useContext(RepoFilesBrowserIdContext);
}
