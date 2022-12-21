import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';
import { Link, useLocation, useParams } from 'react-router-dom';

import { ReactComponent as CreateHoverIcon } from '../../assets/images/create-hover.svg';
import { ReactComponent as CreateIcon } from '../../assets/images/create.svg';
import { ReactComponent as InfoHoverIcon } from '../../assets/images/info-hover.svg';
import { ReactComponent as InfoIcon } from '../../assets/images/info.svg';
import { ReactComponent as LockedHoverIcon } from '../../assets/images/locked-hover.svg';
import { ReactComponent as LockedIcon } from '../../assets/images/locked.svg';
import { ReactComponent as UnlockedHoverIcon } from '../../assets/images/unlocked-hover.svg';
import { ReactComponent as UnlockedIcon } from '../../assets/images/unlocked.svg';
import { buttonReset } from '../../styles/mixins/buttons';
import { allStates } from '../../styles/mixins/hover';
import { Repo } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

export const RepoItem = memo<{ repo: Repo; isActive: boolean }>(
  ({ repo, isActive }) => {
    const theme = useTheme();
    const webVault = useWebVault();
    const lockRepo = useCallback(
      (repo: Repo) => {
        webVault.reposLockRepo(repo.id);
      },
      [webVault]
    );

    return (
      <li>
        <div
          className={cx(
            css`
              display: flex;
              align-items: center;
              height: 36px;
              padding: 0 0 0 25px;

              &:hover {
                background-color: ${theme.colors.hover};
              }
            `,
            isActive &&
              css`
                background-color: ${theme.colors.hover};
              `
          )}
        >
          {repo.state === 'Locked' ? (
            <Link
              to={`/repos/${repo.id}`}
              className={css`
                width: 36px;
                height: 36px;
                display: flex;
                justify-content: center;
                align-items: center;
                flex-shrink: 0;
                margin-right: 7px;
              `}
            >
              <LockedIcon
                className={css`
                  a:hover > & {
                    display: none;
                  }
                `}
              />
              <LockedHoverIcon
                className={css`
                  display: none;

                  a:hover > & {
                    display: inline;
                  }
                `}
              />
            </Link>
          ) : (
            <button
              className={css`
                ${buttonReset}
                width: 36px;
                height: 36px;
                flex-shrink: 0;
                margin-right: 7px;
              `}
              onClick={() => lockRepo(repo)}
            >
              <div
                className={css`
                  display: flex;
                  justify-content: center;
                  align-items: center;
                `}
              >
                <UnlockedIcon
                  className={css`
                    button:hover > div > & {
                      display: none;
                    }
                  `}
                />
                <UnlockedHoverIcon
                  className={css`
                    display: none;

                    button:hover > div > & {
                      display: inline;
                    }
                  `}
                />
              </div>
            </button>
          )}
          <Link
            to={`/repos/${repo.id}`}
            className={cx(
              css`
                text-decoration: none;
                flex-grow: 1;
                font-size: 14px;
                font-weight: normal;
                height: 32px;
                display: flex;
                flex-direction: row;
                align-items: center;
                overflow: hidden;

                ${allStates} {
                  color: ${theme.colors.text};
                  text-decoration: none;
                }
              `,
              isActive &&
                css`
                  font-weight: 600;
                `
            )}
          >
            <span
              className={css`
                text-overflow: ellipsis;
                white-space: nowrap;
                overflow: hidden;
              `}
            >
              {repo.name}
            </span>
          </Link>
          <Link
            to={`/repos/${repo.id}/info`}
            className={css`
              width: 36px;
              height: 36px;
              display: flex;
              justify-content: center;
              align-items: center;
              flex-shrink: 0;
            `}
          >
            <InfoIcon
              className={css`
                a:hover > & {
                  display: none;
                }
              `}
            />
            <InfoHoverIcon
              className={css`
                display: none;

                a:hover > & {
                  display: inline;
                }
              `}
            />
          </Link>
        </div>
      </li>
    );
  }
);

export const RepoCreateItem = memo<{ isActive: boolean }>(({ isActive }) => {
  const theme = useTheme();

  return (
    <li>
      <Link
        to="/repos/create"
        className={cx(
          css`
            display: flex;
            align-items: center;
            height: 36px;
            padding: 0 0 0 25px;

            ${allStates} {
              color: ${theme.colors.text};
              text-decoration: none;
            }

            &:hover {
              background-color: ${theme.colors.hover};
            }
          `,
          isActive &&
            css`
              background-color: ${theme.colors.hover};
            `
        )}
      >
        <div
          className={css`
            width: 36px;
            height: 36px;
            display: flex;
            justify-content: center;
            align-items: center;
            flex-shrink: 0;
            margin-right: 7px;
          `}
        >
          <CreateIcon
            className={css`
              div:hover > & {
                display: none;
              }
            `}
          />
          <CreateHoverIcon
            className={css`
              display: none;

              div:hover > & {
                display: inline;
              }
            `}
          />
        </div>
        <div
          className={cx(
            css`
              text-decoration: none;
              font-size: 14px;
              flex-grow: 1;
              font-weight: normal;
              text-overflow: ellipsis;
              white-space: nowrap;
              overflow: hidden;
            `,
            isActive &&
              css`
                font-weight: 600;
              `
          )}
        >
          Create new
        </div>
      </Link>
    </li>
  );
});

export const Repos = memo(() => {
  const location = useLocation();
  const params = useParams();
  const paramsRepoId: string | undefined = params.repoId;
  const repos = useSubscribe(
    (v, cb) => v.reposSubscribe(cb),
    (v) => v.reposData,
    []
  );

  return (
    <div>
      <ul
        className={css`
          list-style: none;
          margin: 0 0 30px;
          padding: 0;
        `}
      >
        {repos.repos.map((repo) => (
          <RepoItem
            key={repo.id}
            repo={repo}
            isActive={paramsRepoId !== undefined && repo.id === paramsRepoId}
          />
        ))}
        <RepoCreateItem isActive={location.pathname === '/repos/create'} />
      </ul>
    </div>
  );
});
