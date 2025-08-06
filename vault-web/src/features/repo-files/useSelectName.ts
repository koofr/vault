import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { repoFilesLink } from './selectors';

export function useSelectName(
  repoId: string | undefined,
  encryptedPath: string | undefined,
) {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  let name = searchParams.get('name') ?? undefined;

  if (name === '') {
    name = undefined;
  }

  useEffect(() => {
    if (repoId !== undefined && name !== undefined) {
      navigate(repoFilesLink(repoId, encryptedPath), { replace: true });
    }
  }, [navigate, repoId, encryptedPath, name]);

  return name;
}
