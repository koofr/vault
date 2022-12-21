import { cx } from '@emotion/css';
import { DetailedHTMLProps, HTMLAttributes, memo } from 'react';

import { imgRetina } from '../styles/mixins/images';

export const RetinaImage = memo<
  DetailedHTMLProps<HTMLAttributes<HTMLDivElement>, HTMLDivElement> & {
    image: string;
    image2x: string;
    width: number;
    height: number;
    classNameForSize?: (width: number, height: number) => string;
  }
>(
  ({
    image,
    image2x,
    width,
    height,
    classNameForSize,
    className,
    ...props
  }) => {
    return (
      <div
        className={cx(
          imgRetina(image, image2x, width, height),
          classNameForSize !== undefined
            ? classNameForSize(width, height)
            : undefined,
          className
        )}
        {...props}
      />
    );
  }
);
