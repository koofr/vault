import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { Button } from '../../components/Button';
import { LoadingCircle } from '../../components/LoadingCircle';
import { PasswordInput } from '../../components/PasswordInput';
import { TextInput } from '../../components/TextInput';
import { DashboardLayout } from '../../components/dashboard/DashboardLayout';
import { useSingleNavbarBreadcrumb } from '../../components/navbar/useSingleNavbarBreadcrumb';
import { useIsMobile } from '../../components/useIsMobile';
import { buttonReset } from '../../styles/mixins/buttons';
import { RepoCreated, RepoCreateForm } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { RemoteFilesBreadcrumbs } from '../remote-files/RemoteFilesBreadcrumbs';
import { RemoteFilesDirPickerModal } from '../remote-files/RemoteFilesDirPickerModal';
import { RepoConfigInfo } from '../repo/RepoConfigInfo';

const Label = styled.label`
  display: inline-block;
  display: inline-block;
  font-weight: 600;
  font-size: 13px;
  margin-bottom: 2px;
`;

const Group = styled.div`
  margin-bottom: 10px;
`;

const rcloneConfigFormat = `Format:

[name]
type=crypt
remote=rcloneremote:/path
password=obscured password
password2=obscured salt`;

export const RepoCreateFormComponent = memo<{ form: RepoCreateForm }>(
  ({ form }) => {
    const isMobile = useIsMobile();
    const theme = useTheme();
    const webVault = useWebVault();
    const [advancedVisible, setAdvancedVisible] = useState(false);
    const [rcloneConfig, setRcloneConfig] = useState('');
    const locationDirPickerSelect = useCallback(() => {
      webVault.repoCreateLocationDirPickerSelect();
    }, [webVault]);
    const locationDirPickerCancel = useCallback(() => {
      webVault.repoCreateLocationDirPickerCancel();
    }, [webVault]);
    const locationDirPickerCanCreateDir = useCallback(
      (name: string) => webVault.repoCreateLocationDirPickerCanCreateDir(name),
      [webVault]
    );
    const locationDirPickerCreateDir = useCallback(
      (name: string) => {
        webVault.repoCreateLocationDirPickerCreateDir(name);
      },
      [webVault]
    );
    const onSubmit = useCallback(
      (event: React.FormEvent) => {
        event.preventDefault();

        webVault.repoCreateCreate();
      },
      [webVault]
    );
    const canSubmit = form.canCreate && form.createStatus.type !== 'Loading';

    return (
      <div
        className={
          isMobile
            ? css`
                padding: 0 15px;
              `
            : undefined
        }
      >
        <h1
          className={css`
            font-size: 32px;
            font-weight: normal;
            margin: 0 0 20px;
          `}
        >
          Create a new Safe Box
        </h1>
        <form onSubmit={onSubmit}>
          <div
            className={cx(
              css`
                display: flex;
              `,
              isMobile
                ? css`
                    flex-direction: column;
                  `
                : css`
                    flex-direction: row;
                  `
            )}
          >
            <div
              className={cx(
                css`
                  flex-shrink: 0;
                `,
                isMobile
                  ? css`
                      width: 100%;
                    `
                  : css`
                      width: 370px;
                      margin-right: 70px;
                    `
              )}
            >
              {form.createStatus.type === 'Error' ? (
                <div
                  className={css`
                    background-color: #fbedeb;
                    padding: 6px 15px;
                    border-radius: 3px;
                    margin-bottom: 15px;
                  `}
                >
                  {form.createStatus.error}
                </div>
              ) : null}

              <Group>
                <Label>Location</Label>
                <button
                  type="button"
                  className={css`
                    ${buttonReset}
                    text-align: left;
                    border: 1px solid ${theme.colors.borderDark};
                    border-radius: 3px;
                    display: inline-block;
                    padding: 9px 10px 8px;
                    font-size: 14px;
                    width: 100%;
                    cursor: pointer;

                    &:focus {
                      border-color: ${theme.colors.primary};
                    }
                  `}
                  onClick={() => {
                    webVault.repoCreateLocationDirPickerShow();
                  }}
                >
                  {form.locationBreadcrumbs.length > 0 ? (
                    <RemoteFilesBreadcrumbs
                      breadcrumbs={form.locationBreadcrumbs}
                    />
                  ) : (
                    <span>Select a folder</span>
                  )}
                </button>
              </Group>

              <Group>
                <Label
                  htmlFor="password"
                  className={css`
                    margin-bottom: 5px;
                    display: inline-block;
                  `}
                >
                  Safe Key
                </Label>
                <PasswordInput
                  value={form.password}
                  placeholder="Must be at least 8 characters long"
                  onChange={(password) =>
                    webVault.repoCreateSetPassword(password)
                  }
                  inputClassName={css`
                    display: block;
                    width: 100%;
                    font-size: 14px;
                  `}
                  inputId="password"
                />
              </Group>

              {advancedVisible ? (
                <Group>
                  <Label
                    htmlFor="salt"
                    className={css`
                      margin-bottom: 5px;
                      display: inline-block;
                    `}
                  >
                    Salt
                  </Label>
                  <TextInput
                    as="textarea"
                    id="salt"
                    value={form.salt ?? ''}
                    className={css`
                      display: block;
                      width: 100%;
                      height: 180px;
                      margin-bottom: 10px;
                    `}
                    onChange={(event) =>
                      webVault.repoCreateSetSalt(
                        event.currentTarget.value.length > 0
                          ? event.currentTarget.value.trim()
                          : undefined
                      )
                    }
                  />
                  <div
                    className={css`
                      background-color: #d1ecf1;
                      border-radius: 3px;
                      padding: 12px 15px;
                      font-size: 13px;
                      margin-bottom: 15px;
                    `}
                  >
                    <p
                      className={css`
                        margin: 0 0 15px;
                      `}
                    >
                      Salt is used in the key derivation process to create a
                      unique encryption key and helps to protect against
                      potential attacks. It will be stored on the Koofr servers
                      in a secure manner.
                    </p>

                    <p
                      className={css`
                        margin: 0 0 15px;
                      `}
                    >
                      A random Salt has been generated for you. If you prefer,
                      you can leave the Salt field empty, and the default salt
                      will be used (same as in rclone). However, it is
                      recommended to use a unique salt for enhanced security.
                      Using a unique salt helps to increase the complexity of
                      the encryption process, making it more difficult for
                      potential attackers to access the encrypted data.
                    </p>

                    <p
                      className={css`
                        margin: 0;
                      `}
                    >
                      If you wish to transfer the encrypted files to another
                      service, it is necessary to also export the salt,
                      otherwise you won't be able to decrypt your files.
                    </p>
                  </div>
                </Group>
              ) : null}

              <div
                className={css`
                  margin: 15px 0 25px;
                `}
              >
                <Button
                  type="submit"
                  variant={canSubmit ? 'primary' : 'disabled'}
                  disabled={!canSubmit}
                >
                  Create
                </Button>
              </div>

              {!advancedVisible ? (
                <div>
                  <Button
                    type="button"
                    variant="primary-inline"
                    className={css`
                      padding: 0;
                    `}
                    onClick={() => setAdvancedVisible(true)}
                  >
                    Show advanced settings
                  </Button>
                </div>
              ) : null}
            </div>

            {advancedVisible ? (
              <div
                className={
                  isMobile
                    ? css`
                        width: 100%;
                      `
                    : css`
                        width: 400px;
                      `
                }
              >
                <h3
                  className={css`
                    font-size: 28px;
                    font-weight: normal;
                    margin: 23px 0 20px;
                  `}
                >
                  From rclone config
                </h3>
                <pre
                  className={css`
                    background-color: #f8f8f8;
                    color: ${theme.colors.textLight};
                    border-radius: 3px;
                    font-size: 13px;
                    padding: 6px 15px;
                  `}
                >
                  <code>{rcloneConfigFormat}</code>
                </pre>
                {form.fillFromRcloneConfigError !== undefined ? (
                  <div
                    className={css`
                      background-color: #fbedeb;
                      padding: 6px 15px;
                      border-radius: 3px;
                      margin-bottom: 15px;
                    `}
                  >
                    {form.fillFromRcloneConfigError}
                  </div>
                ) : null}
                <div
                  className={css`
                    margin-bottom: 15px;
                  `}
                >
                  <TextInput
                    as="textarea"
                    value={rcloneConfig}
                    className={css`
                      display: block;
                      width: 100%;
                      height: 250px;
                      font-size: 13px;
                    `}
                    onChange={(event) =>
                      setRcloneConfig(event.currentTarget.value)
                    }
                  />
                </div>
                <Button
                  type="button"
                  variant="primary"
                  onClick={() => {
                    webVault.repoCreateFillFromRcloneConfig(rcloneConfig);
                  }}
                >
                  Fill
                </Button>
              </div>
            ) : null}
          </div>
        </form>

        <RemoteFilesDirPickerModal
          dirPickerId={form.locationDirPickerId}
          canSelect={form.locationDirPickerCanSelect}
          select={locationDirPickerSelect}
          cancel={locationDirPickerCancel}
          canShowCreateDir={form.locationDirPickerCanShowCreateDir}
          canCreateDir={locationDirPickerCanCreateDir}
          createDir={locationDirPickerCreateDir}
        />
      </div>
    );
  }
);

export const RepoCreatedComponent = memo<{ created: RepoCreated }>(
  ({ created }) => {
    const theme = useTheme();
    const navigate = useNavigate();
    const [configSaved, setConfigSaved] = useState(false);
    const openRepo = useCallback(() => {
      if (created !== undefined) {
        navigate(`/repos/${created.repoId}`);
      }
    }, [navigate, created]);

    return (
      <div>
        <h1
          className={css`
            font-size: 28px;
            font-weight: normal;
            margin: 0 0 20px;
          `}
        >
          Your Safe Box has been created.
        </h1>
        <p
          className={css`
            margin: 0 0 20px;
          `}
        >
          Before you start using your Safe Box please safely store the
          configuration.
        </p>
        <div
          className={css`
            border-bottom: 1px solid ${theme.colors.border};
            margin-bottom: 25px;
          `}
        />
        <div
          className={css`
            margin-bottom: 25px;
          `}
          onMouseDown={() => setConfigSaved(true)}
        >
          <RepoConfigInfo config={created.config} />
        </div>
        <div
          className={css`
            border-bottom: 1px solid ${theme.colors.border};
            margin-bottom: 25px;
          `}
        />
        <Button
          type="button"
          variant={configSaved ? 'primary' : 'disabled'}
          disabled={!configSaved}
          onClick={openRepo}
        >
          Continue
        </Button>
      </div>
    );
  }
);

export const RepoCreate = memo(() => {
  const webVault = useWebVault();
  const info = useSubscribe(
    (v, cb) => v.repoCreateInfoSubscribe(cb),
    (v) => v.repoCreateInfoData,
    []
  );
  useMemo(() => webVault.repoCreateInit(), [webVault]);
  useEffect(() => {
    return () => {
      webVault.repoCreateReset();
    };
  }, [webVault]);
  const navbarHeader = useSingleNavbarBreadcrumb('Create a new Safe Box');

  return (
    <DashboardLayout navbarHeader={navbarHeader}>
      {info === undefined ||
      info.form?.initStatus.type === 'Initial' ||
      info.form?.initStatus.type === 'Loading' ? (
        <LoadingCircle />
      ) : info?.form !== undefined ? (
        <RepoCreateFormComponent form={info.form} />
      ) : info?.created !== undefined ? (
        <RepoCreatedComponent created={info.created} />
      ) : null}
    </DashboardLayout>
  );
});
