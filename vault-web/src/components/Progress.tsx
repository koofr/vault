import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

export const Progress = memo<{
  percentage: number;
  severity?: 'Normal' | 'Warn' | 'Critical';
}>(({ percentage, severity = 'Normal' }) => {
  const theme = useTheme();
  const backgroundColor =
    severity === 'Critical'
      ? theme.colors.destructive
      : severity === 'Warn'
      ? theme.colors.warning
      : theme.colors.successful;

  return (
    <div
      className={css`
        width: 100%;
        height: 1px;
        background: ${theme.colors.border};
        margin-top: 2px;
        margin-bottom: 2px;
      `}
    >
      <div
        className={css`
          width: 0;
          height: 5px;
          position: relative;
          top: -2px;
        `}
        style={{ width: `${Math.min(percentage, 100)}%`, backgroundColor }}
      ></div>
    </div>
  );
});
