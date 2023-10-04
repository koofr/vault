import { css, cx } from '@emotion/css';
import { memo, MutableRefObject, useCallback, useState } from 'react';

import { useAutofocus } from '../utils/useAutoFocus';

import { ShowPassword } from './ShowPassword';
import { TextInput } from './TextInput';

export interface PasswordInputProps {
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
  className?: string;
  inputRef?: MutableRefObject<HTMLInputElement | null>;
  inputClassName?: string;
  inputId?: string;
  inputAriaLabel?: string;
}

export const PasswordInput = memo<PasswordInputProps>(
  ({
    value,
    placeholder,
    onChange,
    className,
    inputRef,
    inputClassName,
    inputId,
    inputAriaLabel,
  }) => {
    const [isPasswordVisible, setPasswordVisible] = useState(false);
    const togglePasswordVisible = useCallback(
      () => setPasswordVisible((value) => !value),
      [],
    );

    return (
      <div
        className={cx(
          css`
            display: flex;
            flex-direction: row;
            position: relative;
            align-items: center;
          `,
          className,
        )}
      >
        <TextInput
          type={isPasswordVisible ? 'text' : 'password'}
          id={inputId}
          name="password"
          value={value}
          placeholder={placeholder}
          onChange={(event) => onChange(event.currentTarget.value)}
          ref={inputRef}
          className={cx(
            css`
              font-size: 16px;
              width: 250px;
              padding-right: 38px;
            `,
            inputClassName,
          )}
          aria-label={inputAriaLabel}
        />
        <ShowPassword
          value={isPasswordVisible}
          onClick={togglePasswordVisible}
        />
      </div>
    );
  },
);

export const AutoFocusPasswordInput = memo<
  Omit<PasswordInputProps, 'inputRef'>
>((props) => {
  const inputRef = useAutofocus();

  return <PasswordInput {...props} inputRef={inputRef} />;
});
