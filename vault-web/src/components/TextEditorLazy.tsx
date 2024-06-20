import React, { lazy } from 'react';
import { type TextEditorProps } from './TextEditor';

export const TextEditorLazy = lazy<React.FC<TextEditorProps>>(() =>
  import('./TextEditor').then((mod) => ({ default: mod.TextEditor })),
);
