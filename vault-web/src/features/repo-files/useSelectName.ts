import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { repoFilesLink } from './selectors';

export function useSelectName(repoId: string, path: string | undefined) {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  let name = searchParams.get('name') ?? undefined;

  if (name === '') {
    name = undefined;
  }

  useEffect(() => {
    if (name !== undefined) {
      navigate(repoFilesLink(repoId, path), { replace: true });
    }
  }, [navigate, repoId, path, name]);

  return name;
}
