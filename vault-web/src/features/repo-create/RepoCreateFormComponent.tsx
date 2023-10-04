import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import { memo, useCallback, useState } from 'react';

import { Button } from '../../components/Button';
import { PasswordInput } from '../../components/PasswordInput';
import { TextInput } from '../../components/TextInput';
import { useIsMobile } from '../../components/useIsMobile';
import { buttonReset } from '../../styles/mixins/buttons';
import { RepoCreateForm } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { RemoteFilesBreadcrumbs } from '../remote-files/RemoteFilesBreadcrumbs';
import { RemoteFilesDirPickerModal } from '../remote-files/RemoteFilesDirPickerModal';
import { LoadingCircle } from '../../components/LoadingCircle';

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

export const RepoCreateFormComponent = memo<{
  createId: number;
  form: RepoCreateForm;
}>(({ createId, form }) => {
  const isMobile = useIsMobile();
  const theme = useTheme();
  const webVault = useWebVault();
  const [advancedVisible, setAdvancedVisible] = useState(false);
  const [rcloneConfig, setRcloneConfig] = useState('');
  const locationDirPickerOnClick = useCallback(
    (_: number, itemId: string, isArrow: boolean) =>
      webVault.repoCreateLocationDirPickerClick(createId, itemId, isArrow),
    [webVault, createId],
  );
  const locationDirPickerSelect = useCallback(() => {
    webVault.repoCreateLocationDirPickerSelect(createId);
  }, [webVault, createId]);
  const locationDirPickerCancel = useCallback(() => {
    webVault.repoCreateLocationDirPickerCancel(createId);
  }, [webVault, createId]);
  const locationDirPickerCreateDir = useCallback(() => {
    webVault.repoCreateLocationDirPickerCreateDir(createId);
  }, [webVault, createId]);
  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();

      webVault.repoCreateCreateRepo(createId);
    },
    [webVault, createId],
  );
  const canSubmit = form.canCreate && form.createRepoStatus.type !== 'Loading';

  if (
    form.createLoadStatus.type === 'Initial' ||
    form.createLoadStatus.type === 'Loading'
  ) {
    return <LoadingCircle />;
  }

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
                `,
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
                  `,
            )}
          >
            {form.createRepoStatus.type === 'Error' ? (
              <div
                className={css`
                  background-color: #fbedeb;
                  padding: 6px 15px;
                  border-radius: 3px;
                  margin-bottom: 15px;
                `}
              >
                {form.createRepoStatus.error}
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
                  webVault.repoCreateLocationDirPickerShow(createId);
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
                  webVault.repoCreateSetPassword(createId, password)
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
                      createId,
                      event.currentTarget.value.length > 0
                        ? event.currentTarget.value.trim()
                        : undefined,
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
                    unique encryption key and helps to protect against potential
                    attacks. It will be stored on the Koofr servers in a secure
                    manner.
                  </p>

                  <p
                    className={css`
                      margin: 0 0 15px;
                    `}
                  >
                    A random Salt has been generated for you. If you prefer, you
                    can leave the Salt field empty, and the default salt will be
                    used (same as in rclone). However, it is recommended to use
                    a unique salt for enhanced security. Using a unique salt
                    helps to increase the complexity of the encryption process,
                    making it more difficult for potential attackers to access
                    the encrypted data.
                  </p>

                  <p
                    className={css`
                      margin: 0;
                    `}
                  >
                    If you wish to transfer the encrypted files to another
                    service, it is necessary to also export the salt, otherwise
                    you won't be able to decrypt your files.
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
                  webVault.repoCreateFillFromRcloneConfig(
                    createId,
                    rcloneConfig,
                  );
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
        onClick={locationDirPickerOnClick}
        canSelect={form.locationDirPickerCanSelect}
        select={locationDirPickerSelect}
        cancel={locationDirPickerCancel}
        createDirEnabled={form.locationDirPickerCreateDirEnabled}
        createDir={locationDirPickerCreateDir}
      />
    </div>
  );
});
