import { type TextEditorProps } from './TextEditor';
import { lazyLoadingComponent } from './lazyLoadingComponent';

export const TextEditorLazy = lazyLoadingComponent<TextEditorProps>(
  () => import('./TextEditor').then((mod) => mod.TextEditor),
  false
);
