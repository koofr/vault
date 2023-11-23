import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { repoFilesLink } from './selectors';

export function useSelectName(
  repoId: string,
  encryptedPath: string | undefined,
) {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  let name = searchParams.get('name') ?? undefined;

  if (name === '') {
    name = undefined;
  }

  useEffect(() => {
    if (name !== undefined) {
      navigate(repoFilesLink(repoId, encryptedPath), { replace: true });
    }
  }, [navigate, repoId, encryptedPath, name]);

  return name;
}
